use crate::{Chain, Command};

use std::error::Error;

pub struct ImportWalletCmd{
    name : String,
    chain : Chain,
}

impl ImportWalletCmd {
    pub fn new(name : &String, chain : &Chain) -> ImportWalletCmd {
        return ImportWalletCmd{name: name.clone(), chain: *chain }
    }
}

impl Command for ImportWalletCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        return Ok(());
    }
}