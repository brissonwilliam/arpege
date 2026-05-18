use log;
use serde::Deserialize;
use std::sync::OnceLock;
use std::{fs::OpenOptions, io::Read, path::Path};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    pub transcode_bitrate_kbs: u32, // fmpeg arg for transcode bitrate
    pub transcode_codec: String,    // [aac,mp3,flac,opus,wav,alac]
    pub transcode_enabled: bool,
    pub duplicates_reimport: bool,
    pub data_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transcode_bitrate_kbs: 192,
            transcode_codec: String::from("aac"),
            transcode_enabled: true,
            duplicates_reimport: false,
            data_dir: "./data".to_owned(),
        }
    }
}

static CFG_PATH: &str = "data/config.yaml";

impl Config {
    pub fn read() -> Result<Config, Box<dyn std::error::Error>> {
        log::info!("loading config from {}", CFG_PATH);

        let yamlpath = Path::new(CFG_PATH);
        let mut file = OpenOptions::new()
            .create(true) // create if not exists
            .write(true)
            .read(true)
            .open(yamlpath)?;

        let mut file_str = String::new();
        let n = file.read_to_string(&mut file_str)?;

        // file did not exist
        let mut cfg: Config;
        if n == 0 {
            log::info!("{CFG_PATH} is empty, filling defaults");
            cfg = Config::default();
            serde_yaml::to_writer(file, &cfg)?;
        } else {
            cfg = serde_yaml::from_str(file_str.as_str())?;
            set_missing_defaults(&mut cfg);
        }

        log::info!("loaded config {:?}", cfg);

        return Ok(cfg);
    }
}

fn set_missing_defaults(cfg: &mut Config) {
    let def = Config::default();
    if cfg.transcode_codec == "" {
        cfg.transcode_codec = def.transcode_codec;
    }
    if cfg.transcode_bitrate_kbs == 0 {
        cfg.transcode_bitrate_kbs = def.transcode_bitrate_kbs;
    }
    if cfg.data_dir == "" {
        cfg.data_dir = def.data_dir;
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

// Returns the application config. Panics if the config cannot be read. Will read config file
// once if it's not already loaded
pub fn get() -> &'static Config {
    CONFIG.get_or_init(|| {
        Config::read().unwrap_or_else(|e| {
            log::error!("FATAL: failed to load config {CFG_PATH}. Error: {e}");
            std::process::exit(-1);
        })
    })
}
