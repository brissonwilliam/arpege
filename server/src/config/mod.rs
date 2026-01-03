use log;
use std::sync::OnceLock;
use std::{fs::OpenOptions, io::Read, path::Path};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    pub transcode_bitrate_kbs: u32, // fmpeg arg for transcode bitrate
    pub transcode_codec: String,    // [aac,mp3,flac,opus,wav,alac]
    pub transcode_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            transcode_bitrate_kbs: 192,
            transcode_codec: String::from("aac"),
            transcode_enabled: true,
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
        let cfg: Config;
        if n == 0 {
            log::info!("{CFG_PATH} is empty, filling defaults");
            cfg = Config::default();
            serde_yaml::to_writer(file, &cfg)?;
        } else {
            cfg = serde_yaml::from_str(file_str.as_str())?;
        }

        return Ok(cfg);
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
