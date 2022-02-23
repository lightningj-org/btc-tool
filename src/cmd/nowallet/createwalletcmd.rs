extern crate rand;

use std::error::Error;

use bdk::keys::bip39::{Language, Mnemonic};
use cli_table::print_stdout;
use string_error::into_err;

#[warn(unused_imports)]
use crate::{Chain, Command};
use crate::{Settings, WalletData};
use crate::cmd::nowallet::{create_online_wallet, gen_seed_word_table};
use crate::core::password::read_verified_password;
use crate::core::settings::get_or_create_app_dir;
use crate::core::walletdata::{get_wallet_db_path, WALLET_DATA_POSTFIX, WALLET_DB_POSTFIX};

/// Command to create a wallet with given name. Will generate the seed words and display
/// them on screen.
pub struct CreateWalletCmd{
    settings : Settings,
    name : String,
    chain : Chain,
}

impl CreateWalletCmd {
    pub fn new(settings : Settings, name : &String, chain : &Chain) -> CreateWalletCmd {
        return CreateWalletCmd{settings, name: name.clone(), chain: *chain }
    }
}

impl Command for CreateWalletCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        let app_dir = get_or_create_app_dir()?;
        if wallet_exists(&self.name)? {
          return Err(into_err(format!("Error wallet {} already exists, remove files {}{} and {}{} in directory {}.",
                                      &self.name,&self.name,WALLET_DATA_POSTFIX,
                                      &self.name,WALLET_DB_POSTFIX, app_dir.to_str().unwrap())));
        }
        println!("You are about to generate a new wallet with name {}.",&self.name);
        println!();
        println!("First select a password to protect the wallet.");
        println!("It is *VERY IMPORTANT* to remember this password in order recreate this wallet later");
        println!("using the seed phrases.");

        let password = read_verified_password()?;

        println!("New Seed generated:");
        let mut rng = rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng,
                                                  Language::English,
                                                  24)
            .map_err(|_| bdk::Error::Generic("Mnemonic generation error".to_string()))?;
        let words : Vec<&'static str> = mnemonic.word_iter().collect();

        print_stdout(gen_seed_word_table(&words))?;

        println!("\nNote down seed phrase and keep it somewhere safe.");

        create_online_wallet(&self.name, &self.chain,
                             mnemonic, password,
                             &self.settings , &app_dir)?;
        return Ok(());
    }

}

pub(crate) fn wallet_exists(name : &String) ->  Result<bool,Box<dyn std::error::Error>>{
    let exist_wallet = WalletData::exists(name)?;
    let wallet_db_path = get_wallet_db_path(name)?;
    let exists_db = wallet_db_path.exists();
    Ok(exist_wallet || exists_db)
}
