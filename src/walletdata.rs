use std::fs::File;
use std::{fs, io, str};
use std::io::{Error, Write};
use std::path::PathBuf;
use bdk::bitcoin::{Network, PrivateKey};
use bdk::bitcoin::util::bip32::ExtendedPrivKey;
use bdk::{KeychainKind, Wallet};
use bdk::bitcoin::util::key::Error::Base58;
use bdk::bitcoin::util::psbt::PsbtParseError::Base64Encoding;
use bdk::blockchain::Blockchain;
use bdk::database::{BatchDatabase, Database};
use pbkdf2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Pbkdf2
};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use pbkdf2::password_hash::Salt;
use rand_core::RngCore;
use string_error::{into_err, new_err};

use crate::settings::get_or_create_app_dir;

/// Wallet file postfix '.wallet'
static WALLET_DATA_POSTFIX : &str=".wallet";

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
}

impl WalletData {

    /// Creates a new WalletData from given online wallet.
    ///
    /// # Arguments
    /// * name: the name of the wallet.
    /// * wallet: the wallet to build the wallet data structure from
    /// * priv_key: the related private key.
    ///
    pub fn new<B,D>(name: String, wallet: &Wallet<B, D>, priv_key: &PrivateKey) -> WalletData
    where
      B : Blockchain,
      D : BatchDatabase, {
        WalletData{
            name,
            xpriv: priv_key.to_wif(),
            external_descriptor: wallet.get_descriptor_for_keychain(KeychainKind::External).to_string(),
            internal_descriptor: wallet.get_descriptor_for_keychain(KeychainKind::Internal).to_string(),
            network: wallet.network()
        }
    }

    /// Creates a new WalletData from given offline wallet.
    ///
    /// # Arguments
    /// * name: the name of the wallet.
    /// * wallet: the wallet to build the wallet data structure from
    /// * priv_key: the related private key.
    ///
    pub fn new_offline<D>(name: String, wallet: &Wallet<(), D>, priv_key: &PrivateKey) -> WalletData
        where
            D : BatchDatabase, {
        WalletData{
            name,
            xpriv: priv_key.to_wif(),
            external_descriptor: wallet.get_descriptor_for_keychain(KeychainKind::External).to_string(),
            internal_descriptor: wallet.get_descriptor_for_keychain(KeychainKind::Internal).to_string(),
            network: wallet.network()
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
}

/// Help method to retrieve the file path to wallet with given name
fn get_wallet_path(name : &String) ->  Result<PathBuf,Box<dyn std::error::Error>>{
    let mut target_file = get_or_create_app_dir()?;
    target_file.push(format!("{}{}",name,WALLET_DATA_POSTFIX));
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
    let plaintext = cipher.decrypt(nonce, enc_data.as_ref()).map_err(|err| into_err(format!("Error decrypting wallet data: {}",err)))?;
    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use bdk::bitcoin::Network;
    use bdk::blockchain::ElectrumBlockchain;
    use bdk::database::MemoryDatabase;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_new() {
        // Setup
        let priv_key = PrivateKey::from_wif("cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG").unwrap();
        let wallet= Wallet::new(
            "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u",
            Some("wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9"),
            Network::Testnet,
            MemoryDatabase::default(),
            ElectrumBlockchain::from(bdk::electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap())).unwrap();
        // When
        let wallet_data = WalletData::new("test1".to_string(), &wallet, &priv_key);

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
        let priv_key = PrivateKey::from_wif("cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG").unwrap();
        let wallet = Wallet::new_offline(
            "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u",
            Some("wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9"),
            Network::Testnet,
            MemoryDatabase::default()).unwrap();
        // When
        let wallet_data = WalletData::new_offline("test1".to_string(), &wallet, &priv_key);

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
        let wallet_name = "test123".to_string();
        let password = "foo123".to_string();
        let wallet_path = get_wallet_path(&wallet_name).unwrap();
        fs::remove_file(&wallet_path);
        let wallet_data = gen_wallet_data("test123".to_string());
        // Verify that exists returns wallet if file does not exist.
        assert!(!WalletData::exists(&wallet_name).unwrap());
        // Save wallet
        wallet_data.save(&password);
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
    fn test_encrypt_decrypt(){
        let result = encrypt("teest1".to_string(),&"foo123".to_string()).expect("Error test AES encryption");
        let plain_text = decrypt(result, &"foo123".to_string()).unwrap();
        assert_eq!(plain_text, "teest1".to_string())
    }

    /// Help method to generate a populated WalletData
    fn gen_wallet_data(name: String) -> WalletData {
        let priv_key = PrivateKey::from_wif("cQbJPGrjG62SL7z1gRDE7eNjkcuUZYK2TT1MsD3mBfKD5xiooJXG").unwrap();
        let wallet = Wallet::new_offline(
            "wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/0/0/*)#t05p4h3u",
            Some("wpkh([c77e62a6/84'/1'/0']tpubDCAQ9F8i3jjydhAPapH2XWjjfj4RAc9HBefQHBbLhgiCxKdQdzwRLY7eUEoY3KjjsFM5brW5RPrSnDbxgXv6S4ZNv8nSWxbkndDYgCBxf2U/1/0/*)#mf7s98y9"),
            Network::Testnet,
            MemoryDatabase::default()).unwrap();
        return WalletData::new_offline(name, &wallet, &priv_key);
    }
}