use std::env;
use config::{ConfigError, Config, File, FileFormat};
use std::path::{PathBuf};
use bdk::blockchain::{AnyBlockchain, ElectrumBlockchain};
use bdk::database::{AnyDatabase};
use bdk::database::any::SledDbConfiguration;
use bdk::sled;
use crate::core::walletdata::{WALLET_DB_POSTFIX};

/// Structure containing application configurations
/// for application.
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// If debug output should be done.
    pub debug: bool,
    /// The Electrum Connect URL to connect to.
    pub electrum_url: String,
}

/// Default configuration that is written config file if not exists.
static DEFAULT_CONFIG: &str = "#Configuration for btc tool

#if debug mode should be used for more verbose output
debug: false

#The Electrum Connect URL to connect to.
electrum_url: ssl://electrum.blockstream.info:60002
";

/// Name of configuration file
static CONFIG_FILE_NAME: &str = "btc-tool.yml";

/// Environment variable to home directory, of not set is default app dir used.
pub static ENV_VAR_BTC_TOOL_HOME: &str = "BTC_TOOL_HOME";

/// Default path to application directory if not environment variable
/// BTC_TOOL_HOME is set.
static DEFAULT_APP_DIR_PATH : &str =".btc-tool";

impl Settings {

    /// Method to read or create settings file. If file
    /// doesn't exist a new one with default values will be written
    /// to app home directory.
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

    /// Method to return the configured Wallet Blockchain to use.
    pub fn get_wallet_blockchain(self : &Self) -> Result<AnyBlockchain, ConfigError> {
        let blockchain = ElectrumBlockchain::from(bdk::electrum_client::Client::new(&self.electrum_url)
            .map_err(|_| ConfigError::Message(format!("Couldn't initialize Electrum Blockchain using url: {}.", &self.electrum_url)))?);
        let any_blockchain = AnyBlockchain::from(blockchain);
        Ok(any_blockchain)
    }

    /// Method To return the configured Wallet Database to use.
    pub fn get_wallet_database(self : &Self, name : &String) -> Result<AnyDatabase, Box<dyn std::error::Error>> {
        let mut wallet_db_dir = get_or_create_app_dir().map_err(|_| ConfigError::Message("Error reading application home directory".to_string()))?;
        let wallet_db_name = format!("{}{}",name,WALLET_DB_POSTFIX);
        wallet_db_dir.push(&wallet_db_name);
        let sled_config = SledDbConfiguration{
            path: wallet_db_dir.to_str().unwrap().to_string(),
            tree_name: wallet_db_name
        };
        let sled_tree = sled::open(&sled_config.path)?.open_tree(&sled_config.tree_name)?;
        let any_database = AnyDatabase::Sled(sled_tree);
        Ok(any_database)
    }

}

/// Help method to retrieve the configuration file path.
fn get_config_file_path() -> Result<PathBuf,Box<dyn std::error::Error>> {
    let mut app_dir = get_or_create_app_dir()?;
    app_dir.push(CONFIG_FILE_NAME);
    Ok(app_dir)
}

/// Help method to retrieve the applocation home directory.
/// I not environment variable ENV_VAR_BTC_TOOL_HOME is set
pub fn get_or_create_app_dir() -> Result<PathBuf,Box<dyn std::error::Error>>{
    let mut target ;
    if env::var(ENV_VAR_BTC_TOOL_HOME).is_ok() {
        target = PathBuf::from(env::var(ENV_VAR_BTC_TOOL_HOME).unwrap());
    }else {
        target= match home::home_dir(){
            None => PathBuf::from("."),
            Some(home_dir) => home_dir,
        };
        target.push(DEFAULT_APP_DIR_PATH);
    }
    if !std::path::Path::new(target.as_path()).exists() {
        std::fs::create_dir(target.as_path())?;
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use std::fs;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_new() {
        // Setup Remove existing file
        let conf_file = PathBuf::from(format!("target/tmp/{}",CONFIG_FILE_NAME));
        env::set_var(ENV_VAR_BTC_TOOL_HOME, "target/tmp");
        let _ = fs::remove_dir_all("target/tmp");
        // When creating new setting, verify that default values are set
        let settings = Settings::new().unwrap();
        // Then
        assert_eq!(settings.debug, false);
        assert_eq!(settings.electrum_url, "ssl://electrum.blockstream.info:60002");
        assert!(conf_file.exists());
        // When writing new content to settings is it read from file
        fs::write(conf_file, "
debug: true
electrum_url: http://someurl
").unwrap();
        let settings = Settings::new().unwrap();
        // Then
        assert_eq!(settings.debug, true);
        assert_eq!(settings.electrum_url, "http://someurl");
        // Cleanup
        let _ = fs::remove_file(format!("target/tmp/{}",CONFIG_FILE_NAME));
    }
}