use std::error::Error;

use bdk::wallet::AddressIndex;

use crate::{Command, Settings};
use crate::cmd::wallet::{get_wallet, sync_wallet};

/// Sub Command to create a new receive address.
pub struct NewAddressCmd {
    pub settings : Settings,
    pub name : String
}

impl NewAddressCmd {
    pub fn new(settings : Settings, name : &String) -> NewAddressCmd {
        return NewAddressCmd{settings, name: name.clone()}
    }
}

impl Command for NewAddressCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        let (wallet, _) = get_wallet(&self.name, &self.settings)?;
        let _ = sync_wallet(&wallet)?;
        let new_address = wallet.get_address(AddressIndex::New)?;
        println!("New address: {}", new_address.to_string());
        Ok(())
    }
}