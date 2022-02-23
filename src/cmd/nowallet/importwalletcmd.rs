use std::error::Error;
use std::io::stdin;
use std::str::FromStr;
use bdk::keys::bip39::{Language, Mnemonic};
use cli_table::print_stdout;
use string_error::into_err;

use crate::{Chain, Command, Settings};
use crate::cmd::nowallet::createwalletcmd::wallet_exists;
use crate::cmd::nowallet::{create_online_wallet, gen_seed_word_table};
use crate::core::get_confirmation;
use crate::core::password::read_verified_password;
use crate::core::settings::get_or_create_app_dir;
use crate::core::walletdata::{WALLET_DATA_POSTFIX, WALLET_DB_POSTFIX};

/// Command to recreate a wallet with given name. The user will be requested to enter  the seed words
/// and in combination with the password is the wallet recreated. It is important that the same
/// password is used as when initially created the wallet.
pub struct ImportWalletCmd{
    settings : Settings,
    name : String,
    chain : Chain,
}

impl ImportWalletCmd {
    pub fn new(settings : Settings, name : &String, chain : &Chain) -> ImportWalletCmd {
        return ImportWalletCmd{settings, name: name.clone(), chain: *chain }
    }
}

impl Command for ImportWalletCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        let app_dir = get_or_create_app_dir()?;
        if wallet_exists(&self.name)? {
            return Err(into_err(format!("Error wallet {} already exists, remove files {}{} and {}{} in directory {}.",
                                        &self.name,&self.name,WALLET_DATA_POSTFIX,
                                        &self.name,WALLET_DB_POSTFIX, app_dir.to_str().unwrap())));
        }

        println!("You are about to recreate a new wallet with name {}.",&self.name);
        println!("The wallet will be recreated with your seed phrases in combination");
        println!("with your wallet password.");
        println!();
        let mut words: Vec<String>;
        loop {
            println!("Enter your seed phrases (Use Ctrl-C to abort): ");

            words = vec![];
            for n in 1..25 {
                let word = get_word( n)?;
                words.push(word)
            }

            println!();
            println!("You have entered the following seed phrases:");
            print_stdout(gen_seed_word_table(&words))?;
            if get_confirmation("Is this correct? (yes,no):")? {
                break;
            }
        }
        let password = read_verified_password()?;

        let word_string = words.join(" ");
        let mnemonic = Mnemonic::from_str(word_string.as_str())?;

        create_online_wallet(&self.name, &self.chain,
                             mnemonic, password,
                             &self.settings , &app_dir)?;

        return Ok(());
    }
}

fn get_word(n : i32) -> Result<String, Box<dyn Error>>{
    let mut retval;
    loop {
        println!("Enter word {}: ", n);
        let mut word = String::new();
        let _ = stdin().read_line(&mut word)?;
        retval = word.to_lowercase().trim().to_string();
        let matching_words = Language::English.words_by_prefix(retval.as_str());
        if matching_words.len() == 1 && matching_words[0].eq(retval.as_str()){
            break;
        }else{
            println!("Invalid word {} entered, try again",n)
        }
    }

    return Ok(retval);
}