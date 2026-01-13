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

fn log_elapsed(start: std::time::Instant) {
    log::info!("processed in {}ms", start.elapsed().as_millis());
}

impl ProcessorJob {
    pub fn process(&self, store: &storage::Storage) {
        let start = std::time::Instant::now();

        let path = self.path.as_str();
        log::info!("processing {}", path);

        let probemd = match processor::probe::probe(path) {
            Ok(probemd) => probemd,
            Err(err) => return,
        };

        // TODO: parse to another db storage struct
        if probemd.streams.len() < 1 {
            log::error!("no streams on {}, cannot process", path);
            return;
        }

        // TODO: Query db, skip if exists

        // TODO: use internal db struct instead
        if self.should_transcode(probemd) {
            match processor::transcode::transcode(path) {
                Ok(_) => (),
                Err(err) => return,
            }
        } else {
            log::info!("skipping transcode on {}", path);
            log_elapsed(start);
        }


        log_elapsed(start);
    }

    // returns false if transcoding is disabled OR the codec on probe matches the configured codec
    // and bitrate is under allowed limit
    fn should_transcode(&self, probemd: ProbeData) -> bool {
        let cfg = config::get();
        if cfg.transcode_enabled {
            return false;
        }

        // should've been validated earlier
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

        let brate_str = match &probemd.format.bit_rate {
            Some(c) => c,
            None => {
                log::error!("undefined bitrate on {}", probemd.format.filename);
                return false;
            }
        };

        let brate_kbs: u64 = brate_str.parse().unwrap_or(u64::MAX) / 1024;
        let same_codec = cfg.transcode_codec == (*codec).to_lowercase().trim();
        return !(same_codec && brate_kbs <= cfg.transcode_bitrate_kbs as u64);
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
