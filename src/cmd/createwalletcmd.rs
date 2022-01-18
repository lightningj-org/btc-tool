#[warn(unused_imports)]
use crate::{Chain, Command};

use std::error::Error;
use std::str::FromStr;
use bdk::bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use bdk::keys::bip39::{Language, Mnemonic};
use bdk::keys::{DerivableKey};
use bdk::{KeychainKind};
use bdk::{
    blockchain::ElectrumBlockchain,
    bitcoin::Network,
    electrum_client::Client,
    Wallet,
};
use bdk::database::SqliteDatabase;
use bdk::bitcoin::Network::Testnet;
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::blockchain::noop_progress;
use bdk::descriptor::IntoWalletDescriptor;
use bdk::wallet::AddressIndex;
use cli_table::{Cell, print_stdout, Table, TableStruct, WithTitle};
use cli_table::format::Border;

extern crate rand;

pub struct CreateWalletCmd{
    name : String,
    chain : Chain,
}

impl CreateWalletCmd {
    pub fn new(name : &String, chain : &Chain) -> CreateWalletCmd {
        return CreateWalletCmd{name: name.clone(), chain: *chain }
    }
}

impl Command for CreateWalletCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        println!("New Seed generated:");
        let mut rng = rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng,
                                                  Language::English,
                                                  24)
            .map_err(|_| bdk::Error::Generic("Mnemonic generation error".to_string()))?;
        let words : Vec<&'static str> = mnemonic.word_iter().collect();

        print_stdout(gen_seed_word_table(&words))?;

        println!("\nNote down seed phrase and keep it somewhere safe.");


        let network = Testnet;
        let ext_path = DerivationPath::from_str("m/84'/1'/0'/0/0").unwrap();
        println!("Ext Derivation Path: {:?}", ext_path);
        let seed = mnemonic.to_seed("foo123");
        let root_key = ExtendedPrivKey::new_master(network, &seed).unwrap();

        let wif_key = root_key.private_key.to_wif();
        println!("PrivateKey WIF: {}",&wif_key);

        //let rootKey2 = ExtendedPrivKey::from_str(wif_key.as_str());

        let ext_key = (root_key, ext_path);
        let (ext_descriptor, _, _) = bdk::descriptor!(wpkh(ext_key)).unwrap();

        let int_path = DerivationPath::from_str("m/84'/1'/0'/1/0").unwrap();
        let int_key = (root_key, int_path);
        let (int_descriptor, _, _) = bdk::descriptor!(wpkh(int_key)).unwrap();
        //let secp =Secp256k1::new();
       // let (extWallet, keyMap) = externalDescriptor.into_wallet_descriptor(&secp,Network::Testnet).unwrap();

        println!("External Descriptor: {}", ext_descriptor);
        println!("Internal Descriptor: {}", int_descriptor);


        //println!("Descriptor: {}",desc);
        //println!("Keys: {:?}",keys);
        //println!("networks: {:?}", networks);

        let wallet: Wallet<ElectrumBlockchain, SqliteDatabase> = Wallet::new(
            ext_descriptor,
            Some(int_descriptor),
            Network::Testnet,
            SqliteDatabase::new("testname.db".to_string()),
            ElectrumBlockchain::from(bdk::electrum_client::Client::new("ssl://electrum.blockstream.info:60002").unwrap()),
        )?;

        wallet.sync(noop_progress(), None)?;

        let address = wallet.get_address(AddressIndex::New)?;
        println!("Generated Address: {}", address);

        let ext2desc = wallet.get_descriptor_for_keychain(KeychainKind::External);
        println!("ext2desc: {}", ext2desc);

        let balance = wallet.get_balance()?;
        println!("Current Balance: {}", balance);

        return Ok(());
    }

}

/// Help method to generate a seed word table with justified columns.
fn gen_seed_word_table(words : &Vec<&'static str>) -> TableStruct {
    vec![
        vec![format!(" 1: {}",words[0]).cell(),format!(" 2: {}",words[1]).cell(),format!(" 3: {}",words[2]).cell(),format!(" 4: {}",words[3]).cell(),format!(" 5: {}",words[4]).cell(),format!(" 6: {}",words[5]).cell()],
        vec![format!(" 7: {}",words[6]).cell(),format!(" 8: {}",words[7]).cell(),format!(" 9: {}",words[8]).cell(),format!("10: {}",words[9]).cell(),format!("11: {}",words[10]).cell(),format!("12: {}",words[11]).cell()],
        vec![format!("13: {}",words[12]).cell(),format!("14: {}",words[13]).cell(),format!("15: {}",words[14]).cell(),format!("16: {}",words[15]).cell(),format!("17: {}",words[16]).cell(),format!("18: {}",words[17]).cell()],
        vec![format!("19: {}",words[18]).cell(),format!("20: {}",words[19]).cell(),format!("21: {}",words[20]).cell(),format!("22: {}",words[21]).cell(),format!("23: {}",words[22]).cell(),format!("24: {}",words[23]).cell()],
    ].table()
}