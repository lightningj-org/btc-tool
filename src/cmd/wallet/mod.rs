use bdk::blockchain::{noop_progress};
use bdk::{SyncOptions, TransactionDetails};
use cli_table::{Cell, CellStruct, Style, Table, TableStruct};
use string_error::new_err;

use crate::{PrivateKey, read_password, Settings, WalletData};
use crate::core::walletcontainer::WalletContainer;

pub mod getbalancecmd;
pub mod newaddresscmd;
pub mod listtransactionscmd;
pub mod sendcmd;

/// Help method to retrieve a wallet container and private key of
/// Wallet with given name.
pub fn get_wallet(name : &String, settings: &Settings) ->  Result<(WalletContainer,PrivateKey),Box<dyn std::error::Error>> {
    let password = read_password("Enter Password")?;
    let wallet_data =  WalletData::load(name,&password)?;
    if !wallet_data.online {
        return Err(new_err("Invalid wallet type, expected online wallet but found offline wallet."));
    }
    let retval = wallet_data.to_wallet(settings)?;
    Ok(retval)
}


/// Help method to synchronize an online wallet.
///
pub fn sync_wallet(wallet : &WalletContainer) -> Result<(),Box<dyn std::error::Error>>{
    if wallet.is_online() {
        println!("Synchronizing Blockchain...");
        let (online_wallet, blockchain) = wallet.get_online_wallet()?;
        online_wallet.sync(blockchain, SyncOptions {
            progress: Some(Box::new(noop_progress())),
        })?;
        println!("Sync Complete.");
        println!();
    }
    Ok(())
}

/// Help method to generate a seed word table with justified columns.
pub fn gen_transaction_table(transactions : &Vec<TransactionDetails>) -> TableStruct {
    let mut rows : Vec<Vec<CellStruct>> = vec![];
    for transaction in transactions{
        rows.push(vec![
            transaction.txid.to_string().cell(),
            transaction.sent.to_string().cell(),
            transaction.received.to_string().cell(),
            match &transaction.fee {
                None => "None".to_string(),
                Some(fee_value) => format!("{}", fee_value)
            }.cell(),
            match &transaction.confirmation_time {
                None => "None".to_string(),
                Some(confirm_time) => format!("{}", confirm_time.height)
            }.cell(),
        ])
    }
    return rows.table().title(vec![
        "TransactionId".cell().bold(true),
        "Sent".cell().bold(true),
        "Received".cell().bold(true),
        "Fee".cell().bold(true),
        "Confirmation Block".cell().bold(true),
    ])
}
