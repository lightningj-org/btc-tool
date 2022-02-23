use bdk::blockchain::AnyBlockchain;
use bdk::database::AnyDatabase;
use std::error::Error;
use bdk::{TransactionDetails, Wallet};
use bdk::wallet::{AddressIndex, AddressInfo};
use string_error::into_err;
use crate::core::walletcontainer::WalletType::{Offline, Online};

/// Internal enum indicating type of wallet used.
enum WalletType {
    Online(Wallet<AnyBlockchain,AnyDatabase>),
    Offline(Wallet<(),AnyDatabase>),
}

/// WalletContainer is a struct to give one interface to handle both
/// online and offline wallets from commands allicable to both types.
pub struct WalletContainer{
    typ : WalletType,
}


impl WalletContainer {

    /// Creates a new WalletContainer for an online wallet.
    pub fn new_online(wallet : Wallet<AnyBlockchain,AnyDatabase>) -> WalletContainer{
        return WalletContainer{typ : Online(wallet)}
    }

    /// Creates a new WalletContainer for an offline wallet.
    pub fn new_offline(wallet : Wallet<(),AnyDatabase>) -> WalletContainer{
        return WalletContainer{typ : Offline(wallet)}
    }

    /// Returns true if the underlying wallet is an online wallet.
    pub fn is_online(&self) -> bool{
        return match &self.typ {
            Online(_) => true,
            Offline(_) => false,
        }
    }

    /// Returns the underlying online wallet or error if wallet is of offline type
    pub fn get_online_wallet(&self) -> Result<&Wallet<AnyBlockchain,AnyDatabase>,Box<dyn Error>>{
        if let Online(wallet) =  &self.typ {
            return Ok(wallet);
        }
        return Err(into_err("Error tried to access online wallet methods for offline wallet.".to_string()));
    }

    /// Returns the underlying offline wallet or error if wallet is of online type
    pub fn _get_offline_wallet(&self) -> Result<&Wallet<(),AnyDatabase>,Box<dyn Error>>{
        if let Offline(wallet) =  &self.typ {
            return Ok(wallet);
        }
        return Err(into_err("Error tried to access offline wallet methods for online wallet.".to_string()));
    }

    /// Wrapper function for calling both online and offline wallet variant with same method call.
    /// see Wallet get_balance documentation for details.
    pub fn get_balance(&self) -> Result<u64, bdk::Error>{
        return match &self.typ {
            Online(wallet) => wallet.get_balance(),
            Offline(wallet) => wallet.get_balance(),
        };
    }

    /// Wrapper function for calling both online and offline wallet variant with same method call.
    /// see Wallet get_address documentation for details.
    pub fn get_address(&self, address_index: AddressIndex) -> Result<AddressInfo, bdk::Error> {
        return match &self.typ {
            Online(wallet) => wallet.get_address(address_index),
            Offline(wallet) => wallet.get_address(address_index)
        };
    }

    /// Wrapper function for calling both online and offline wallet variant with same method call.
    /// see Wallet list_transactions documentation for details.
    pub fn list_transactions(&self, include_raw: bool) -> Result<Vec<TransactionDetails>, bdk::Error>{
        return match &self.typ {
            Online(wallet) => wallet.list_transactions(include_raw),
            Offline(wallet) => wallet.list_transactions(include_raw)
        };
    }
}

