use crate::{config, storage};
use std::process::ExitStatus;
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

        let cfg = config::get();
        if cfg.transcode_enabled {
            self.transcode(path);
        }

        log::info!("job completed in {}ms", start.elapsed().as_millis());
    }

    fn transcode(&self, path: &str) {
        let start = std::time::Instant::now();

        let cfg = config::get();
        let kbs = cfg.transcode_bitrate_kbs.to_string() + "k";
        let args: Vec<&str> = [
            // input file
            "-i",
            path,
            // segment into small chunks of 8 seconds
            "-f",
            "segment",
            "-segment_time",
            "8",
            // reduce logging
            "-loglevel",
            "warning", // [quiet, panic, error, warning, info, verbose, debug, trace]
            // there may be a video stream containing the album art, keep it!
            "-c:v",
            "copy",
            // transcode audio to aac
            "-c:a",
            "aac",
            "-b:a",
            kbs.as_str(),
            "-movflags",
            "+faststart",
            "data/fs/out_%03d.m4a", // todo: smarter split, write into uuid given by db
        ]
        .to_vec();

        log::info!("invoking 'ffmpeg {}'", args.join(" "));

        let cmd = match std::process::Command::new("ffmpeg")
            .args(args)
            .env("PATH", "/bin")
            .spawn()
        {
            Ok(res) => res,
            Err(err) => {
                log::error!("error running ffmpeg command: {}", err.to_string());
                return;
            }
        };
        log::info!("spawned child process {}", cmd.id());

        // todo: await with timeout. Use tokio sleep and cmd.try_wait() to poll output
        let cmd_out = match cmd.wait_with_output() {
            Ok(res) => res,
            Err(err) => {
                log::error!("error waiting for ffmpeg to end: {}", err.to_string());
                return;
            }
        };
        log::info!("transcode completed in {}ms", start.elapsed().as_millis());

        if !ExitStatus::success(&cmd_out.status) {
            log::error!("ffmpeg cmd exitted with status {}", cmd_out.status);
            return;
        }
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

