use crate::config;

// TODO: make these configurable
const SEGMENT_TIME_SECS: &str = "8";
const THREADS: &str = "6";
const DATA_FOLDER: &str = "data/fs/transcodes";

pub fn transcode(path: &str) -> Result<(), std::io::Error> {
    let start = std::time::Instant::now();

    let cfg = config::get();

    // todo: support multi resolution quality
    let kbs = cfg.transcode_bitrate_kbs.to_string() + "k";

    let output = String::from(DATA_FOLDER) + "/out_%03d.m4a";

    let args: Vec<&str> = [
        // input file
        "-i",
        path,
        // segment into small chunks of 8 seconds
        "-f",
        "segment",
        "-segment_time",
        SEGMENT_TIME_SECS,
        // reduce logging
        "-loglevel",
        "warning", // [quiet, panic, error, warning, info, verbose, debug, trace]
        // no video
        "-vn",
        // threads
        "-threads",
        THREADS,
        // transcode audio to aac
        "-c:a",
        "aac",
        // constant bitrate
        "-b:a",
        kbs.as_str(),
        // moovflag must be added to embed metadata on each chunk for streaming
        // faststart = move md to the begining of chunk
        // frag_keyframe = fragment at each keyframe
        // empty_moov = empty header for faster streams
        "-movflags",
        "+faststart+frag_keyframe+empty_moov+default_base_moof",
        // output
        output.as_str(), // todo: smarter split, write into uuid given by db
    ]
    .to_vec();

    log::info!(
        "transcoding {} | invoking 'ffmpeg {}'",
        path,
        args.join(" ")
    );

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
