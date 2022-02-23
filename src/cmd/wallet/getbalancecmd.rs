use std::error::Error;

use crate::{Command, Settings};
use crate::cmd::wallet::{get_wallet, sync_wallet};

/// Subcommand use to show the current balance of a wallet.
pub struct GetBalanceCmd{
    pub settings : Settings,
    pub name : String
}

impl GetBalanceCmd {
    pub fn new(settings : Settings, name : &String) -> GetBalanceCmd {
        return GetBalanceCmd{settings, name: name.clone()}
    }
}

impl Command for GetBalanceCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        let (wallet, _) = get_wallet(&self.name, &self.settings)?;
        let _ = sync_wallet(&wallet)?;
        println!("Current balance: {}", wallet.get_balance()?);
        Ok(())
    }
}