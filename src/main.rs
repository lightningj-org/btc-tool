extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod settings;
mod cmd;
mod walletdata;

use std::fmt::Debug;
use cmd::command::Command;
use settings::Settings;
use clap::{ArgEnum, AppSettings, Parser, Subcommand};
use crate::cmd::createwalletcmd::CreateWalletCmd;
use crate::cmd::importwalletcmd::ImportWalletCmd;

// Btc Tool TODO
#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct NoWalletCli {
    #[clap(subcommand)]
    command: NoWalletCommands,
}

#[derive(Subcommand)]
enum NoWalletCommands {
    /// Create new BTC wallet
    Create {
        /// The name of the wallet
        name: String,
        /// Target Chain of Wallet
        #[clap(short, long, arg_enum,default_value="testnet")]
        chain: Chain,
    },
    /// Import existing wallet from Seed Phrases
    Import {
        /// The name to wallet to import from seed
        name: String,
        /// Target Chain of Wallet
        #[clap(short, long, arg_enum,default_value="testnet")]
        chain: Chain,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum Chain {
    Testnet,
    Mainnet,
}

// TODO Create lib
// TODO Figure out where to put methods

fn get_chain_name(chain : &Chain) -> &str {
    match chain {
        Chain::Testnet=> "testnet",
        Chain::Mainnet=> "mainnet",
    }
}

fn main() {
    let settings = parse_settings();

    println!("Settings debug {} wallet {}", settings.debug,settings.wallet_name);

    let cli = NoWalletCli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    let command  = match &cli.command {
        NoWalletCommands::Create { name, chain } => {
            Box::new(CreateWalletCmd::new(name, chain))as Box<dyn Command>
        },
        NoWalletCommands::Import { name,chain } => {
            Box::new(ImportWalletCmd::new(name, chain)) as Box<dyn Command>
        },
    };
    match command.execute() {
        Err(error) => {
            println!("Error occurred executing command:{}", error.to_string());
            std::process::exit(-3);
        },
        _ => ()
    }

}

fn parse_settings() -> Settings {
    let setting_result = Settings::new();
    let settings = match setting_result {
        Ok(s) =>  s,
        Err(error) => {
            println!("Invalid configuration: {}", error.to_string());
            std::process::exit(-2);
        }
    };
    return settings
}


