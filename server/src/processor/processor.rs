use crate::processor;
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

        let path = self.path.as_str();
        log::info!("processing {}", path);

        let probemd = match processor::probe::probe(path) {
            Ok(probemd) => probemd,
            Err(err) => return,
        };

        let cfg = config::get();
        if cfg.transcode_enabled {
            match processor::transcode::transcode(path) {
                Ok(_) => (),
                Err(err) => return,
            }
        }

        log::info!("job completed in {}ms", start.elapsed().as_millis());
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
