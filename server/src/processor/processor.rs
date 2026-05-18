use crate::processor;
use crate::processor::probe::ProbeData;
use crate::{config, storage};
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Debug)]
pub struct ProcessorJob {
    pub path: String,
}

pub struct JobResult {
    remove_import: bool,
    matches: usize,
    transcoded: bool,
}

fn job_result_abort() -> JobResult {
    return JobResult {
        remove_import: false,
        transcoded: false,
        matches: 0,
    };
}

impl ProcessorJob {
    async fn process(&self, store: &storage::Storage) {
        let start = std::time::Instant::now();

        let jr = self.do_process(store).await;
        if jr.remove_import {
            self.move_import();
        }

        log::info!("processed in {}ms", start.elapsed().as_millis());
    }

    // TODO: maybe add a force reimport config
    // TODO: process input images. Generate different resolutions
    async fn do_process(&self, store: &storage::Storage) -> JobResult {
        let path = self.path.as_str();
        log::info!("processing {}", path);

        // Probe
        let mut probemd = match processor::probe::probe(path) {
            Ok(probemd) => probemd,
            Err(err) => {
                log::error!("could not probe {} because err: {}", path, err.to_string());
                return job_result_abort();
            }
        };
        log::debug!("probed {:?}", probemd);

        // Query db, skip if exists
        let c = match self.new_db_match_criteria(path, &mut probemd) {
            Some(c) => c,
            None => return job_result_abort(),
        };
        let matches = match store.find_meta(c).await {
            Ok(m) => m,
            Err(err) => {
                log::error!("error finding meta match for import: {:?}", err);
                return job_result_abort();
            }
        };
        log::debug!("found {} db matches", matches.len());

        let mut jr = JobResult {
            remove_import: true,
            transcoded: false,
            matches: matches.len(),
        };

        if matches.len() > 0 {
            let cfg = config::get();
            if !cfg.duplicates_reimport {
                log::info!("{path} already exists in library. Skipping (consider duplicates_reimport config)");
                return jr
            } 
            log::info!("{path} already exists in library. Will re-importing (consider duplicates_reimport config)")
        } else {
            write new entry to db!!!
            log::info!("Adding {path} to db");
        }

        // Transcode (maybe)
        match self.transcode(path, &probemd) {
            Some(did_transcode) => jr.transcoded = did_transcode,
            None => return job_result_abort(),
        };

        return jr;
    }

    fn new_db_match_criteria(
        &self,
        path: &str,
        probemd: &mut ProbeData,
    ) -> Option<storage::FindMetaCriteria> {
        // take ownership of first stream value without offsetting / resizing the whole thing
        // TODO: maybe pick the best audio stream ?
        if probemd.streams.len() < 1 {
            log::error!("no streams on {}, cannot process", path);
            return None;
        }
        let probestream = probemd.streams.swap_remove(0);
        let dur = probestream
            .duration
            .unwrap_or("".to_owned())
            .parse::<u32>()
            .unwrap_or(0);

        if probemd.format.tags.is_none() {
            log::error!("no format.tags on {}, cannot process", path);
            return None;
        }

        let tags = probemd.format.tags.as_ref().unwrap();
        // as_deref() converts Option<string> on tag fields to Option<&str>
        let title = tags.title.as_deref().unwrap_or("").to_owned();
        let artist = tags.artist.as_deref().unwrap_or("").to_owned();
        let album = tags.artist.as_deref().unwrap_or("").to_owned();

        let c = storage::FindMetaCriteria::new(title, dur, artist, album);
        return Some(c);
    }

    fn transcode(&self, path: &str, probemd: &ProbeData) -> Option<bool> {
        // ###### Precheck Transcode ###############
        let cfg = config::get();
        if !cfg.transcode_enabled {
            log::info!("transcode disabled, skipping {}", path);
            return Some(false);
        }

        let brate_str = match &probemd.format.bit_rate {
            Some(c) => c,
            None => {
                log::error!("undefined bitrate on {}", probemd.format.filename);
                return None;
            }
        };
        let brate_kbs: u64 = brate_str.parse().unwrap_or(u64::MAX) / 1024;

        let audio = &probemd.streams[0];
        let codec = match &audio.codec {
            Some(c) => c,
            None => {
                log::error!(
                    "could not find codec on audio stream for {}",
                    probemd.format.filename
                );
                return None;
            }
        };
        let same_codec = cfg.transcode_codec == (*codec).to_lowercase().trim();

        if same_codec && brate_kbs <= cfg.transcode_bitrate_kbs as u64 {
            log::info!("bitrate & codec already match, skipping {}", path);
            return Some(false);
        };

        match processor::transcode::transcode(path) {
            Ok(_) => Some(true),
            Err(err) => {
                log::error!("error transcoding {}. err: {}", path, err.to_string());
                return None;
            }
        }
    }

    pub fn move_import(&self) {
        let cfg = config::get();

        let p = &self.path;
        let src_path = std::path::Path::new(p);
        let dst_dir = cfg.data_dir.clone() + "/fs/origin/";
        let dst_dir_path = std::path::Path::new(&dst_dir);

        let file_name = match src_path.file_name() {
            Some(name) => name,
            None => {
                log::error!("invalid source path: {}", p);
                return;
            }
        };

        let dst_path: std::path::PathBuf = dst_dir_path.join(file_name);

        if let Err(e) = std::fs::rename(src_path, &dst_path) {
            log::warn!(
                "rename failed {} -> {:?}: {}. Attempting copy + delete",
                p,
                dst_path,
                e
            );

            // fallback: copy + delete
            if let Err(e) = std::fs::copy(src_path, &dst_path) {
                log::error!("copy failed {} -> {:?}: {}", p, dst_path, e);
                return;
            }

            if let Err(e) = std::fs::remove_file(src_path) {
                log::error!("remove source failed {}: {}", p, e);
                return;
            }
        }
        log::info!("moved {} to {:?}", p, dst_path);
    }
}

pub struct Processor {
    sender: Option<Sender<ProcessorJob>>,
}

impl Processor {
    pub fn new() -> Self {
        return Processor { sender: None };
    }

    pub fn push(&self, job: ProcessorJob) {
        let _ = self.sender.as_ref().unwrap().send(job);
    }

    pub fn start(&mut self, store: storage::Storage) {
        let (tx, rx) = mpsc::channel::<ProcessorJob>();
        let clone = store.clone();

        self.sender = Some(tx);

        tokio::task::spawn(async move {
            for job in rx {
                job.process(&clone).await;
            }
        });
    }
}
