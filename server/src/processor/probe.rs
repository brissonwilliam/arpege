use serde::{Deserialize, Serialize};

/* Full example of probe data
    {
        "streams": [
            {
                "index": 0,
                "codec_name": "aac",
                "codec_long_name": "AAC (Advanced Audio Coding)",
                "profile": "LC",
                "codec_type": "audio",
                "codec_tag_string": "mp4a",
                "codec_tag": "0x6134706d",
                "sample_fmt": "fltp",
                "sample_rate": "44100",
                "channels": 2,
                "channel_layout": "stereo",
                "bits_per_sample": 0,
                "initial_padding": 0,
                "id": "0x1",
                "r_frame_rate": "0/0",
                "avg_frame_rate": "0/0",
                "time_base": "1/44100",
                "start_pts": 2112,
                "start_time": "0.047891",
                "duration_ts": 10735616,
                "duration": "243.438005",
                "bit_rate": "261610",
                "nb_frames": "10484",
                "extradata_size": 2,
                "disposition": {
                    "default": 1,
                    "dub": 0,
                    "original": 0,
                    "comment": 0,
                    "lyrics": 0,
                    "karaoke": 0,
                    "forced": 0,
                    "hearing_impaired": 0,
                    "visual_impaired": 0,
                    "clean_effects": 0,
                    "attached_pic": 0,
                    "timed_thumbnails": 0,
                    "non_diegetic": 0,
                    "captions": 0,
                    "descriptions": 0,
                    "metadata": 0,
                    "dependent": 0,
                    "still_image": 0,
                    "multilayer": 0
                },
                "tags": {
                    "creation_time": "2019-08-24T00:09:50.000000Z",
                    "language": "eng",
                    "vendor_id": "[0][0][0][0]"
                }
            },
            {
                "index": 1,
                "codec_name": "mjpeg",
                "codec_long_name": "Motion JPEG",
                "profile": "Baseline",
                "codec_type": "video",
                "codec_tag_string": "[0][0][0][0]",
                "codec_tag": "0x0000",
                "width": 2000,
                "height": 2000,
                "coded_width": 2000,
                "coded_height": 2000,
                "closed_captions": 0,
                "film_grain": 0,
                "has_b_frames": 0,
                "sample_aspect_ratio": "1:1",
                "display_aspect_ratio": "1:1",
                "pix_fmt": "yuvj420p",
                "level": -99,
                "color_range": "pc",
                "color_space": "bt470bg",
                "chroma_location": "center",
                "refs": 1,
                "id": "0x0",
                "r_frame_rate": "90000/1",
                "avg_frame_rate": "0/0",
                "time_base": "1/90000",
                "start_pts": 4310,
                "start_time": "0.047889",
                "duration_ts": 21909420,
                "duration": "243.438000",
                "bits_per_raw_sample": "8",
                "disposition": {
                    "default": 0,
                    "dub": 0,
                    "original": 0,
                    "comment": 0,
                    "lyrics": 0,
                    "karaoke": 0,
                    "forced": 0,
                    "hearing_impaired": 0,
                    "visual_impaired": 0,
                    "clean_effects": 0,
                    "attached_pic": 1,
                    "timed_thumbnails": 0,
                    "non_diegetic": 0,
                    "captions": 0,
                    "descriptions": 0,
                    "metadata": 0,
                    "dependent": 0,
                    "still_image": 0,
                    "multilayer": 0
                }
            }
        ],
        "format": {
            "filename": "./data/import/ato.m4a",
            "nb_streams": 2,
            "nb_programs": 0,
            "nb_stream_groups": 0,
            "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
            "format_long_name": "QuickTime / MOV",
            "start_time": "0.047889",
            "duration": "243.438005",
            "size": "10032962",
            "bit_rate": "329708",
            "probe_score": 100,
            "tags": {
                "major_brand": "M4A ",
                "minor_version": "0",
                "compatible_brands": "M4A mp42isom",
                "creation_time": "2019-08-24T00:09:50.000000Z",
                "iTunSMPB": " 00000000 00000840 0000029A 0000000000A3C526 00000000 00000000 00000000 00000000 00000000 00000000 00000000 00000000",
                "iTunNORM": " 00000644 00000643 00009EBA 00009690 0001DF01 0001DF01 00007FFF 00007FFF 0000B667 0002E0C8",
                "title": "24",
                "artist": "ATO",
                "album_artist": "ATO",
                "composer": "Jonathon Ng & Ato Alexander",
                "album": "24 - Single",
                "genre": "Hip-Hop/Rap",
                "track": "1/1",
                "disc": "1/1",
                "compilation": "0",
                "gapless_playback": "0",
                "date": "2019-09-18T12:00:00Z",
                "account_id": "brissonwilliam1@hotmail.com",
                "copyright": "\E2\84\97 2019 MCMXCV",
                "rating": "1",
                "media_type": "1",
                "purchase_date": "2019-10-13 01:20:28",
                "sort_name": "24",
                "sort_album": "24 - Single",
                "sort_artist": "ATO"
            }
        }
    }
*/
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
    pub filename: String,
    pub nb_streams: u32,
    pub nb_programs: u32,
    pub nb_stream_groups: u32,
    pub format_name: String,
    pub format_long_name: String,
    pub start_time: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bit_rate: Option<String>,
    pub probe_score: u32,
    pub tags: Option<FormatTags>,
}

#[derive(Debug, Deserialize)]
pub struct FormatTags {
    pub major_brand: Option<String>,
    pub minor_version: Option<String>,
    pub compatible_brands: Option<String>,
    pub creation_time: Option<String>,

    // audio / track metadata
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub composer: Option<String>,
    pub genre: Option<String>,
    pub track: Option<String>, // "1/1"
    pub disc: Option<String>, // "1/1"
    pub date: Option<String>,
    pub copyright: Option<String>,
    pub iTunSMPB: Option<String>,
    pub iTunNORM: Option<String>,
    pub compilation: Option<String>,
    pub gapless_playback: Option<String>,
    pub media_type: Option<String>,
    pub purchase_date: Option<String>,
    pub sort_name: Option<String>,
    pub sort_album: Option<String>,
    pub sort_artist: Option<String>,
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
