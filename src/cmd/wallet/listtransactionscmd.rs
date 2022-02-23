use std::error::Error;
use cli_table::print_stdout;


use crate::{Command, Settings};
use crate::cmd::wallet::{gen_transaction_table, get_wallet, sync_wallet};

/// Subcommand list all transactions created by a wallet in a well formatted ascii table.
pub struct ListTransactionsCmd{
    pub settings : Settings,
    pub name : String
}

impl ListTransactionsCmd {
    pub fn new(settings : Settings, name : &String) -> ListTransactionsCmd {
        return ListTransactionsCmd{settings, name: name.clone()}
    }
}

impl Command for ListTransactionsCmd {

    fn execute(self : &Self) -> Result<(), Box<dyn Error>>{
        let (wallet, _) = get_wallet(&self.name, &self.settings)?;
        let _ = sync_wallet(&wallet)?;
        // In future include raw transactions in list
        let transactions = wallet.list_transactions(false)?;

        let _ = print_stdout(gen_transaction_table(&transactions));

        Ok(())
    }
}