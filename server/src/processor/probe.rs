use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ProbeData {
    pub streams: Vec<Stream>,
    pub format: Format,
}

#[derive(Debug, Deserialize)]
pub struct Stream {
    #[serde(rename = "codec_type")]
    pub codec_type: String, // filter for "audio"

    #[serde(rename = "codec_name")]
    pub codec: Option<String>,

    #[serde(rename = "bit_rate")]
    pub bitrate: Option<String>, // ffprobe reports as string

    pub channels: Option<u32>,

    #[serde(rename = "channel_layout")]
    pub channel_layout: Option<String>, // stereo / mono

    pub duration: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub duration: Option<String>,

    pub tags: Option<Tags>,
}

#[derive(Debug, Deserialize)]
pub struct Tags {
    pub title: Option<String>, // track name
    pub album: Option<String>,
    pub artist: Option<String>,
    pub composer: Option<String>,
    pub track: Option<String>, // e.g. "1/1"
}

pub fn probe(path: &str) -> Result<ProbeData, std::io::Error> {
    let args: Vec<&str> = [
        "-loglevel",
        "warning", // [quiet, panic, error, warning, info, verbose, debug, trace]
        "-show_format",
        "-show_streams",
        "-print_format",
        "json",
        path,
    ]
    .to_vec();

    log::info!("invoking 'ffprobe {}'", args.join(" "));

    let cmd = match std::process::Command::new("ffprobe")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .env("PATH", "/bin")
        .spawn()
    {
        Ok(res) => res,
        Err(err) => {
            log::error!("error running ffprobe : {}", err.to_string());
            return Err(err);
        }
    };
    log::info!("spawned child process {}", cmd.id());

    // todo: await with timeout. Use tokio sleep and cmd.try_wait() to poll output
    let cmd_out = match cmd.wait_with_output() {
        Ok(res) => res,
        Err(err) => {
            log::error!("error waiting for ffprobe to end: {}", err.to_string());
            return Err(err);
        }
    };

    if !std::process::ExitStatus::success(&cmd_out.status) {
        log::error!("ffmpeg cmd exitted with status {}", cmd_out.status);
        let err = std::io::Error::new(
            std::io::ErrorKind::Other,
            "ffprobe exitted with unexpected status code",
        );
        return Err(err);
    }

    let s: String = match String::from_utf8(cmd_out.stdout) {
        Ok(out) => out,
        Err(err) => {
            log::error!(
                "error parsing ffprobe output to utf8 string: {}",
                err.to_string()
            );
            let newerr = std::io::Error::new(std::io::ErrorKind::Other, err);
            return Err(newerr);
        }
    };
    log::debug!("ffprobe output:\n{}", s);

    let md: ProbeData = match serde_json::from_str(s.as_str()) {
        Ok(md) => md,
        Err(err) => {
            log::error!("error deserializing ffprobe json: {}", err.to_string());
            return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
        }
    };

    return Ok(md);
}
