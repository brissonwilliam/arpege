use crate::processor;
use crate::processor::probe::ProbeData;
use crate::{config, storage};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Debug)]
pub struct ProcessorJob {
    pub path: String,
    pub override_existing: bool,
}

impl ProcessorJob {
    pub fn process(&self, store: &storage::Storage) {
        let start = std::time::Instant::now();

        let p = &self.path;
        self.do_process(store);
        self.move_import();

        log::info!("processed in {}ms", start.elapsed().as_millis());
    }

    // processes the job and returns a bool indicate whether or not everything happened as expected
    fn do_process(&self, store: &storage::Storage) -> bool {
        let path = self.path.as_str();
        log::info!("processing {}", path);

        let mut probemd = match processor::probe::probe(path) {
            Ok(probemd) => probemd,
            Err(err) => {
                log::error!("could not probe {}", path);
                return false;
            }
        };
        log::debug!("probed {:?}", probemd);

        if probemd.streams.len() < 1 {
            log::error!("no streams on {}, cannot process", path);
            return false;
        }
        let stream = probemd.streams.swap_remove(0); // take ownership of first stream value without
                                                     // offsetting / resizing the whole thing

        if probemd.format.tags.is_none() {
            log::error!("no format.tags on {}, cannot process", path);
            return false;
        }
        let tags = probemd.format.tags.unwrap();

        // Query db, skip if exists
        let dur = stream
            .duration
            .unwrap_or("".to_owned())
            .parse::<u32>()
            .unwrap_or(0);

        let c = storage::FindMetaCriteria::new(
            tags.title.unwrap_or("".to_owned()),
            dur,
            tags.artist.unwrap_or("".to_owned()),
            tags.album.unwrap_or("".to_owned()),
        );
        let matches = match store.find_meta(c).await {
            Ok(v) => v,
            Err(err) => {
                log::error!("error finding meta match for import: {:?}", err);
                return false;
            }
        };

        // TODO: maybe add a force reimport config

        // TODO: use internal db struct instead

        // ###### Precheck Transcode ###############
        let cfg = config::get();
        if !cfg.transcode_enabled {
            log::info!("transcode disabled, skipping {}", path);
            return true;
        }

        let brate_str = match &probemd.format.bit_rate {
            Some(c) => c,
            None => {
                log::error!("undefined bitrate on {}", probemd.format.filename);
                return false;
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
                return false;
            }
        };
        let same_codec = cfg.transcode_codec == (*codec).to_lowercase().trim();

        if same_codec && brate_kbs <= cfg.transcode_bitrate_kbs as u64 {
            log::info!("bitrate & codec already match, skipping {}", path);
            return true;
        };

        // ########### Transcode ##############
        match processor::transcode::transcode(path) {
            Ok(_) => (),
            Err(err) => {
                log::error!("error transcoding {}", path);
                return false;
            }
        }

        // TODO: ###### Store meta in db #########

        return true;
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

        thread::spawn(move || {
            for job in rx {
                job.process(&clone);
            }
        });
    }
}
