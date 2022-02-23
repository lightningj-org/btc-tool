extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod cmd;
mod core;

use std::error::Error;
use std::fmt::Debug;
use bdk::bitcoin::{Network, PrivateKey};
use cmd::command::Command;
use crate::core::settings::Settings;
use clap::{AppSettings, ArgEnum, Parser, Subcommand};
use crate::cmd::nowallet::createwalletcmd::CreateWalletCmd;
use crate::cmd::nowallet::importwalletcmd::ImportWalletCmd;
use crate::cmd::wallet::getbalancecmd::GetBalanceCmd;
use crate::cmd::wallet::listtransactionscmd::ListTransactionsCmd;
use crate::cmd::wallet::newaddresscmd::NewAddressCmd;
use crate::cmd::wallet::sendcmd::SendCmd;
use crate::core::password::read_password;
use crate::core::walletdata::WalletData;

// Btc Tool TODO
#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct NoWalletCli {
    #[clap(subcommand)]
    command: NoWalletCommands,
}

// Btc Tool TODO
#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct OnlineWalletCli {
    #[clap(subcommand)]
    command: OnlineWalletCommands,
}

#[derive(Subcommand)]
enum NoWalletCommands {
    /// Create new BTC wallet
    Create {
        /// The name of the wallet
        #[clap(short, long, default_value="default")]
        name: String,
        /// Target Chain of Wallet
        #[clap(short, long, arg_enum,default_value="testnet")]
        chain: Chain,
    },
    /// Import existing wallet from Seed Phrases
    Import {
        /// The name to wallet to import from seed
        #[clap(short, long, default_value="default")]
        name: String,
        /// Target Chain of Wallet
        #[clap(short, long, arg_enum,default_value="testnet")]
        chain: Chain,
    },
}

#[derive(Subcommand)]
enum OnlineWalletCommands {
    /// Get Current Balance of Wallet
    GetBalance {
        /// The name of the wallet
        #[clap(short, long, default_value="default")]
        name: String,
    },
    /// Get Current Balance of Wallet
    NewAddress {
        /// The name of the wallet
        #[clap(short, long, default_value="default")]
        name: String,
    },
    /// List all transactions using wallet
    ListTransactions {
        /// The name of the wallet
        #[clap(short, long, default_value="default")]
        name: String,
    },
    /// Sends funds to specified address
    Send {
        /// The name of the wallet
        #[clap(short, long, default_value="default")]
        name: String,
        /// Address to send to.
        #[clap(short, long)]
        address: String,
        /// Amount of to send in satoshis.
        #[clap(short='s', long)]
        amount: u64,
        /// Optional fee in sats/vbyte.
        #[clap(short='f', long, default_value="0.0")]
        fee: f32,
    },
    #[clap(flatten)]
    NoWalletComamnds(NoWalletCommands),
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
pub enum Chain {
    Testnet,
    Mainnet,
}

// TODO Create lib

fn get_chain_name(chain : &Chain) -> Network {
    match chain {
        Chain::Testnet=> Network::Testnet,
        Chain::Mainnet=> Network::Bitcoin,
    }
}

fn main() {
    let settings = parse_settings();

    let wallet_exists = WalletData::exists(&"default".to_string()).expect(format!("Error reading default wallet file.").as_str());
    let command_result = get_command(settings, wallet_exists);
    if command_result.is_err() {
        process_error(command_result.err().unwrap());
    }else {
        match command_result.unwrap().execute() {
            Err(error) => process_error(error),
            _ => ()
        }
    }
}

fn get_command(settings : Settings, wallet_exists : bool) -> Result<Box<dyn Command>,Box<dyn Error>>{
    let command = match wallet_exists {
        true => run_cli(settings)?,
        false => {
            println!("Default wallet doesn't exist. Create or import a default wallet.");
            run_nowallet_cli(settings)?
        }
    };
    Ok(command)
}

fn run_cli(settings : Settings) -> Result<Box<dyn Command>,Box<dyn Error>>{
    let cli = OnlineWalletCli::parse();

    let command  = match &cli.command {
        OnlineWalletCommands::GetBalance { name } => {
            Box::new(GetBalanceCmd::new(settings, name)) as Box<dyn Command>
        },
        OnlineWalletCommands::NewAddress { name } => {
            Box::new(NewAddressCmd::new(settings, name)) as Box<dyn Command>
        },
        OnlineWalletCommands::ListTransactions { name } => {
            Box::new(ListTransactionsCmd::new(settings, name)) as Box<dyn Command>
        },
        OnlineWalletCommands::Send { name ,address, amount, fee} => {
            Box::new(SendCmd::new(settings, name, address, amount, fee)) as Box<dyn Command>
        },
        OnlineWalletCommands::NoWalletComamnds(no_wallet_cmd) => {
            run_nowallet_cmd(settings, no_wallet_cmd)?
        }
    };

    return Ok(command)
}

fn run_nowallet_cmd(settings : Settings, command : &NoWalletCommands) ->
                                                                      Result<Box<dyn Command>,Box<dyn Error>>{
    let command  = match command {
        NoWalletCommands::Create { name, chain } => {
            Box::new(CreateWalletCmd::new(settings, name, chain))as Box<dyn Command>
        },
        NoWalletCommands::Import { name,chain } => {
            Box::new(ImportWalletCmd::new(settings, name, chain)) as Box<dyn Command>
        },
    };

    return Ok(command)
}

fn run_nowallet_cli(settings : Settings) -> Result<Box<dyn Command>,Box<dyn Error>>{
    let cli = NoWalletCli::parse();

    return run_nowallet_cmd(settings, &cli.command)
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

pub fn process_error(error : Box<dyn Error>){
    println!("Error occurred executing command:{}", error.to_string());
    std::process::exit(-3);
}
