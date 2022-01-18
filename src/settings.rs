use config::{ConfigError, Config, File, FileFormat};
use std::path::{PathBuf};


#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub wallet_name: String,
}

static DEFAULT_CONFIG: &str = "#Configuration for btc tool

#if debug mode should be used for more verbose output
debug: false

#Name of wallet used if not specified as argument
wallet_name: default
";

static CONFIG_FILE_NAME: &str= "btc-tool.yml";
static APP_DIR_PATH : &str=".btc-tool";

impl Settings {

    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::from_str(DEFAULT_CONFIG,FileFormat::Yaml))?;

        let config_path = get_config_file_path()
            .map_err(|_| ConfigError::Message(format!("Couldn't get path to application config file.")))?;

        if config_path.exists() {
            s.merge(File::from(config_path).required(false))?;
        }else{
            std::fs::write(config_path, DEFAULT_CONFIG).map_err(|_| ConfigError::Message(format!("Couldn't write default configuration file {}", DEFAULT_CONFIG)))?;
        }

        s.try_into()
    }
}

fn get_config_file_path() -> Result<PathBuf,Box<dyn std::error::Error>> {
    let mut app_dir = get_or_create_app_dir()?;
    app_dir.push(CONFIG_FILE_NAME);
    Ok(app_dir)
}

pub fn get_or_create_app_dir() -> Result<PathBuf,Box<dyn std::error::Error>>{
    let mut target = match home::home_dir(){
        None => PathBuf::from("."),
        Some(home_dir) => home_dir,
    };
    target.push(".btc-tool");
    if !std::path::Path::new(target.as_path()).exists() {
        std::fs::create_dir(target.as_path())?;
    }
    Ok(target)
}