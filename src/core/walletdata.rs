use std::{fs, str};
use std::fs::File;
use std::io::{Write};
use std::path::PathBuf;

use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use bdk::{Wallet};
use bdk::bitcoin::{Network, PrivateKey};
use bdk::database::{AnyDatabase, BatchDatabase};
use pbkdf2::{
    password_hash::{
        PasswordHasher, rand_core::OsRng, SaltString
    },
    Pbkdf2
};
use pbkdf2::password_hash::Salt;
use rand_core::RngCore;
use string_error::{into_err, new_err};

use crate::core::settings::get_or_create_app_dir;
use crate::core::walletcontainer::WalletContainer;
use crate::Settings;

/// Wallet file postfix '.wallet'
pub static WALLET_DATA_POSTFIX : &str=".wallet";
/// Wallet file postfix '.wallet'
pub static WALLET_DB_POSTFIX : &str=".db";

/// WalletData is a wallet specific data structure that
/// is serializable into YAML and is stored into a wallet
/// file encrypted.
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletData {
    // Name of Wallet
    pub name: String,
    // Private Key in WIF String format
    pub xpriv: String,
    // External Descriptor
    pub external_descriptor: String,
    // Internal Descriptor
    pub internal_descriptor: String,
    // The related network
    pub network : Network,
    // IF wallet is an offline or online wallet
    pub online: bool,
}

impl WalletData {

    /// Creates a new WalletData from given online wallet.
    ///
    /// # Arguments
    /// * name: the name of the wallet.
    /// * wallet: the wallet to build the wallet data structure from
    /// * priv_key: the related private key.
    ///
    pub fn new<D>(name: &String, wallet: &Wallet<D>,
                    external_descriptor: &String,
                    internal_descriptor: &String,
                    priv_key: &PrivateKey) -> WalletData
    where
      D : BatchDatabase, {
        WalletData{
            name : name.clone(),
            xpriv: priv_key.to_wif(),
            external_descriptor: external_descriptor.clone(),
            internal_descriptor: internal_descriptor.clone(),
            network: wallet.network(),
            online: true,
        }
    }

    /// Creates a new WalletData from given nowallet wallet.
    ///
    /// # Arguments
    /// * name: the name of the wallet.
    /// * wallet: the wallet to build the wallet data structure from
    /// * priv_key: the related private key.
    ///
    pub fn _new_offline<D>(name: String, wallet: &Wallet<D>,
                           external_descriptor: &String,
                           internal_descriptor: &String,
                           priv_key: &PrivateKey) -> WalletData
        where
            D : BatchDatabase, {
        WalletData{
            name,
            xpriv: priv_key.to_wif(),
            external_descriptor: external_descriptor.clone(),
            internal_descriptor: internal_descriptor.clone(),
            network: wallet.network(),
            online: false,
        }
    }

    /// Method to check if related wallet exists in application
    /// home directory.
    ///
    /// # Arguments
    /// * name: the name of the wallet.
    ///
    pub fn exists(name: &String) -> Result<bool,Box<dyn std::error::Error>>{
        let wallet_path = get_wallet_path(name)?;
        return Ok(wallet_path.exists())
    }


    /// Method to load wallet of given name from encrypted file.
    ///
    /// # Arguments
    /// * name: the name of the wallet.
    /// * password: The encryption password of the wallet data file.
    ///
    pub fn load(name: &String, password: &String) ->  Result<WalletData,Box<dyn std::error::Error>>{
        let load_file = get_wallet_path(name)?;
        let enc_data = fs::read(load_file)?;
        let yaml_data = decrypt(enc_data, password)?;
        let wallet_data: WalletData = serde_yaml::from_str(&yaml_data)?;
        return Ok(wallet_data)
    }

    /// Method to save wallet of given name to encrypted file.
    ///
    /// # Arguments
    /// * password: The encryption password of the wallet data file.
    ///
    pub fn save(self : &Self, password : &String) -> Result<(),Box<dyn std::error::Error>>{
        let yaml_data = serde_yaml::to_string(self)?;
        let enc_data = encrypt(yaml_data, password)?;
        let save_file = get_wallet_path(&self.name)?;

        let mut file = File::create(save_file.as_path())?;
        file.write_all(enc_data.as_slice())?;

        Ok(())
    }

    /// Method to convert a Wallet Data to a Online Wallet and PrivateKey tuple.
    ///
    /// # Arguments
    /// * settings: The application settings.
    ///
    pub fn to_wallet(self : &Self, settings : &Settings) -> Result<(WalletContainer, PrivateKey),Box<dyn std::error::Error>> {

        let database = settings.get_wallet_database(&self.name)?;
        let priv_key = PrivateKey::from_wif(&self.xpriv)?;

        let wallet_container = match &self.online {
            true => {
                let wallet: Wallet<AnyDatabase> = Wallet::new(
                    &self.external_descriptor,
                    Some(&self.internal_descriptor),
                    self.network,
                    database,
                )?;
                WalletContainer::new_online(wallet, settings.get_wallet_blockchain()?)
            }
            false => {
                let wallet: Wallet<AnyDatabase> = Wallet::new(
                    &self.external_descriptor,
                    Some(&self.internal_descriptor),
                    self.network,
                    database,
                )?;
                WalletContainer::new_offline(wallet)
            }
        };

        Ok((wallet_container,priv_key))
    }


}

/// Help method to retrieve the file path to wallet with given name
pub fn get_wallet_path(name : &String) ->  Result<PathBuf,Box<dyn std::error::Error>>{
    let mut target_file = get_or_create_app_dir()?;
    target_file.push(format!("{}{}",name,WALLET_DATA_POSTFIX));
    return Ok(target_file);
}

/// Help method to retrieve the file path to wallet with given name
pub fn get_wallet_db_path(name : &String) ->  Result<PathBuf,Box<dyn std::error::Error>>{
    let mut target_file = get_or_create_app_dir()?;
    target_file.push(format!("{}{}",name,WALLET_DB_POSTFIX));
    return Ok(target_file);
}

/// Help method encrypt serialized wallet data with given password.
fn encrypt(data : String, password : &String) -> Result<Vec<u8>,Box<dyn std::error::Error>>{

    let salt = SaltString::generate(&mut OsRng);
    let gen_key = Pbkdf2.hash_password(password.as_bytes(), &salt).map_err(|err| into_err(format!("Error generating wallet encryption key from password: {}",err)))?;
    let key = gen_key.hash.unwrap();

    let mut nonce_data : [u8 ; 12] = [0;12];
    OsRng.fill_bytes( &mut nonce_data);

    let cipher = Aes256Gcm::new(Key::from_slice(key.as_bytes()));
    let nonce = Nonce::from_slice(&nonce_data);

    let ciphertext = cipher.encrypt(nonce, data.as_bytes()).map_err(|err| into_err(format!("Error encrypting wallet data: {}",err)))?;

    let result =  [salt.as_bytes().to_vec(),
        nonce_data.to_vec(),
        ciphertext].concat();

    Ok(result)
}

/// Help method decrypt serialized wallet data with given password.
fn decrypt(data : Vec<u8>, password : &String) -> Result<String,Box<dyn std::error::Error>>{
    if data.len() < 35 {
        return Err(new_err("Invalid length of encrypted data"));
    }
    let salt_string = str::from_utf8(&data[0..22])?;
    let salt = Salt::new(salt_string).map_err(|err| into_err(format!("Error generating wallet encryption key from password: {}",err)))?;

    let hash = Pbkdf2.hash_password(password.as_bytes(), &salt).map_err(|err| into_err(format!("Error generating wallet decryption key from password: {}",err)))?;
    let key = hash.hash.unwrap();

    let nonce = Nonce::from_slice(&data[22..34]);
    let cipher = Aes256Gcm::new(Key::from_slice(key.as_bytes()));
    let enc_data = &data[34..];
    let plaintext = cipher.decrypt(nonce, enc_data.as_ref()).map_err(|err| into_err(format!("Error decrypting wallet data, was password correct?: {}",err)))?;
    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    use std::env;
    use bdk::bitcoin::Network;
    use bdk::bitcoin::Network::Testnet;
    use bdk::blockchain::ElectrumBlockchain;
    use bdk::database::MemoryDatabase;
    use crate::core::settings::ENV_VAR_BTC_TOOL_HOME;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_new() {
        // Setup
        set_home_dir();
        let priv_key = PrivateKey::from_wif("cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG").unwrap();
        let external_descriptor = "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u".to_string();
        let internal_descriptor = "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9".to_string();
        let wallet= Wallet::new(
            &external_descriptor,
            Some(&internal_descriptor),
            Network::Testnet,
            MemoryDatabase::default()).unwrap();

        // When
        let wallet_data = WalletData::new(&"test1".to_string(), &wallet, &external_descriptor, &internal_descriptor,&priv_key);

        // Then
        assert_eq!(wallet_data.name, "test1".to_string());
        assert_eq!(wallet_data.xpriv, "cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG".to_string());
        assert_eq!(wallet_data.external_descriptor, "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u".to_string());
        assert_eq!(wallet_data.internal_descriptor, "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9".to_string());
        assert_eq!(wallet_data.network, Network::Testnet)
    }

    #[test]
    fn test_new_offline() {
        // Setup
        set_home_dir();
        let priv_key = PrivateKey::from_wif("cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG").unwrap();
        let external_descriptor = "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u".to_string();
        let internal_descriptor = "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9".to_string();
        let wallet = Wallet::new(
            &external_descriptor,
            Some(&internal_descriptor),
            Network::Testnet,
            MemoryDatabase::default()).unwrap();
        // When
        let wallet_data = WalletData::_new_offline("test1".to_string(), &wallet, &external_descriptor, &internal_descriptor, &priv_key);

        // Then
        assert_eq!(wallet_data.name, "test1".to_string());
        assert_eq!(wallet_data.xpriv, "cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG".to_string());
        assert_eq!(wallet_data.external_descriptor, "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u".to_string());
        assert_eq!(wallet_data.internal_descriptor, "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9".to_string());
        assert_eq!(wallet_data.network, Network::Testnet)
    }

    #[test]
    fn test_exists_save_and_load(){
        // setup
        set_home_dir();
        let wallet_name = "test123".to_string();
        let password = "foo123".to_string();
        let wallet_path = get_wallet_path(&wallet_name).unwrap();
        let _ = fs::remove_file(&wallet_path);
        let wallet_data = gen_wallet_data("test123".to_string());
        // Verify that exists returns wallet if file does not exist.
        assert!(!WalletData::exists(&wallet_name).unwrap());
        // Save wallet
        let _ = wallet_data.save(&password);
        // Verify the file exists
        assert!(WalletData::exists(&wallet_name).unwrap());
        // Load the wallet
        let loaded_wallet_data = WalletData::load(&wallet_name,&password).unwrap();
        // Verify the original content and loaded content match
        assert_eq!(wallet_data.name, loaded_wallet_data.name);
        // Cleanup
        fs::remove_file(&wallet_path).unwrap();
    }

    #[test]
    fn test_to_wallet(){
        // setup
        set_home_dir();
        let wallet_name = "test123".to_string();
        let wallet_path = get_wallet_path(&wallet_name).unwrap();
        let _ = fs::remove_file(&wallet_path);
        let wallet_data = gen_wallet_data(wallet_name.clone());
        let settings = gen_settings();
        // When
        let (wallet, private_key) = wallet_data.to_wallet(&settings).unwrap();
        // Then
        assert!(!wallet.is_online());
        assert_eq!(private_key.network, Testnet);
    
    }

    #[test]
    fn test_encrypt_decrypt(){
        let result = encrypt("teest1".to_string(),&"foo123".to_string()).expect("Error test AES encryption");
        let plain_text = decrypt(result, &"foo123".to_string()).unwrap();
        assert_eq!(plain_text, "teest1".to_string())
    }

    /// Help method to generate a populated WalletData
    fn gen_wallet_data(name: String) -> WalletData {
        let priv_key = PrivateKey::from_wif("cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG").unwrap();
        let external_descriptor = "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u".to_string();
        let internal_descriptor = "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9".to_string();
        let wallet = Wallet::new(
            &external_descriptor,
            Some(&internal_descriptor),
            Network::Testnet,
            MemoryDatabase::default()).unwrap();
        return WalletData::_new_offline(name, &wallet, &external_descriptor, &internal_descriptor,&priv_key);
    }

    fn gen_settings() -> Settings{
        return Settings{
            debug: false,
            electrum_url: "".to_string()
        }
    }

    fn set_home_dir(){
        env::set_var(ENV_VAR_BTC_TOOL_HOME, "target/tmp");
    }
}