use crate::config;

pub fn transcode(path: &str) -> Result<(), std::io::Error> {
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
            return Err(err);
        }
    };
    log::info!("spawned child process {}", cmd.id());

    // todo: await with timeout. Use tokio sleep and cmd.try_wait() to poll output
    let cmd_out = match cmd.wait_with_output() {
        Ok(res) => res,
        Err(err) => {
            log::error!("error waiting for ffmpeg to end: {}", err.to_string());
            return Err(err);
        }
    };
    log::info!("transcode completed in {}ms", start.elapsed().as_millis());

    if !std::process::ExitStatus::success(&cmd_out.status) {
        log::error!("ffmpeg cmd exitted with status {}", cmd_out.status);
        let err = std::io::Error::new(
            std::io::ErrorKind::Other,
            "ffmpeg exitted with unexpected status code",
        );
        return Err(err);
    }

    return Ok(());
}
