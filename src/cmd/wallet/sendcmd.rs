use std::error::Error;
use std::str::FromStr;
use bdk::bitcoin::Address;
use bdk::{FeeRate, SignOptions};
use bdk::blockchain::Blockchain;
use cli_table::print_stdout;
use string_error::into_err;

use crate::{Command, Settings};
use crate::cmd::wallet::{gen_transaction_table, get_wallet, sync_wallet};

/// Command to send a specific amout of SAT to a specific address. There is also
/// an optional parameter for fee in SATS/VBytes.
pub struct SendCmd{
    settings : Settings,
    name : String,
    to_address : String,
    amount : u64,
    fee : f32
}

impl SendCmd {
    pub fn new(settings : Settings, name : &String, to_address: &String, amount: &u64, fee: &f32) -> SendCmd {
        return SendCmd{settings, name: name.clone(),
            to_address: to_address.clone(), amount: amount.clone(),
            fee: fee.clone()}
    }
}

impl Command for SendCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        let (wallet, _) = get_wallet(&self.name, &self.settings)?;
        let _ = sync_wallet(&wallet)?;

        let (online_wallet, blockchain) = wallet.get_online_wallet()?;

        let address = Address::from_str(self.to_address.as_str()).map_err(|e| into_err(format!("Invalid address specified: {}",e)))?;
        let mut tx_builder = online_wallet.build_tx();
        tx_builder
            .add_recipient(address.script_pubkey(), self.amount)
            .enable_rbf();
        if self.fee != 0.0{
            tx_builder.fee_rate(FeeRate::from_sat_per_vb(self.fee));
        }
        let (mut psbt, tx_details) = tx_builder.finish()?;

        let _ = print_stdout(gen_transaction_table(&vec![tx_details]));

        let finalized = online_wallet.sign(&mut psbt, SignOptions::default())?;
        if finalized {
            let raw_transaction = psbt.extract_tx();
            blockchain.broadcast(&raw_transaction)?;
            let txid = &raw_transaction.txid();
            println!(
                "Transaction sent to Network.\nExplorer URL: https://blockstream.info/testnet/tx/{txid}",
                txid = txid
            );
        }else{
            println!("Transaction could not be signed.")
        }

        Ok(())
    }
}