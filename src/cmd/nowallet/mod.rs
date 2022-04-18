use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use bdk::bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use bdk::database::AnyDatabase;
use bdk::keys::bip39::Mnemonic;
use bdk::Wallet;
use cli_table::{Cell, Table, TableStruct};
use crate::{Chain, get_chain_name, Settings, WalletData};
use crate::core::walletdata::get_wallet_path;

pub mod createwalletcmd;
pub mod importwalletcmd;

/// Help method to generate a seed word table with justified columns.
pub fn gen_seed_word_table<T : Display>(words : &Vec<T>) -> TableStruct {
    vec![
        vec![format!(" 1: {}",words[0]).cell(),format!(" 2: {}",words[1]).cell(),format!(" 3: {}",words[2]).cell(),format!(" 4: {}",words[3]).cell(),format!(" 5: {}",words[4]).cell(),format!(" 6: {}",words[5]).cell()],
        vec![format!(" 7: {}",words[6]).cell(),format!(" 8: {}",words[7]).cell(),format!(" 9: {}",words[8]).cell(),format!("10: {}",words[9]).cell(),format!("11: {}",words[10]).cell(),format!("12: {}",words[11]).cell()],
    ].table()
}

/// Help method to create an online wallet that is in common for create and import commands.
pub(crate) fn create_online_wallet(name : &String, chain : &Chain,
                                   mnemonic : Mnemonic, password : String,
                                   settings : &Settings, app_dir : &PathBuf) -> Result<(), Box<dyn Error>>{
    let network = get_chain_name(chain);
    let ext_path = DerivationPath::from_str("m/84'/1'/0'/0/0").unwrap();
    let seed = mnemonic.to_seed(&password);
    let root_key = ExtendedPrivKey::new_master(network, &seed).unwrap();

    let ext_key = (root_key, ext_path);
    let (ext_descriptor, ext_key_map, _) = bdk::descriptor!(wpkh(ext_key)).unwrap();

    let ext_descriptor_with_secret = ext_descriptor.to_string_with_secret(&ext_key_map);

    let int_path = DerivationPath::from_str("m/84'/1'/0'/1/0").unwrap();
    let int_key = (root_key, int_path);
    let (int_descriptor, int_key_map, _) = bdk::descriptor!(wpkh(int_key)).unwrap();
    let int_descriptor_with_secret = int_descriptor.to_string_with_secret(&int_key_map);
    let mut wallet_db_path = app_dir.clone();
    wallet_db_path.push(format!("{}.db",name));

    let database = settings.get_wallet_database(name)?;

    let wallet: Wallet<AnyDatabase> = Wallet::new(
        &ext_descriptor_with_secret,
        Some(&int_descriptor_with_secret),
        network,
        database
    )?;

    let wallet_data = WalletData::new(name,&wallet,
                                      &ext_descriptor_with_secret,
                                      &int_descriptor_with_secret,
                                      &root_key.private_key);
    let _ = wallet_data.save(&password)?;
    let wallet_path = get_wallet_path(name)?;
    println!("Wallet created and stored in {}", wallet_path.to_str().unwrap());
    return Ok(())
}