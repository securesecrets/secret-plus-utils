use crate::{
    utils::generate_label,
    cli_types::{GasLog, NetContract},
    secretcli::{test_contract_handle, test_inst_init},
    constants::{GAS, STORE_GAS},
};
use cosmwasm_std::{ContractInfo, Addr};
use serde_json::Result;

/// Pass in the following arguments to automatically create a struct that implements the [Deployable] trait:
///
/// name of the struct, default user (&str), path to deployable wasm (&str), secretd backend key (&str)
///
/// Caller space needs to have imported Deployable and NetContract from secretcli
#[macro_export]
macro_rules! impl_deployable {
    ($x:ident, $user:literal, $deployable_file:literal, $backend:literal) => {
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct $x {
            pub info: NetContract
        }

        impl<'a> Deployable<'a> for $x {
            const DEFAULT_USER: &'a str = $user;
            const BACKEND: &'a str = $backend;
            const DEPLOYABLE_FILE: &'a str = $deployable_file;

            fn get_info(&self) -> &NetContract {
                &self.info
            }
        }
    };
}

pub trait Deployable<'a> {
    const DEFAULT_USER: &'a str;
    const BACKEND: &'a str;
    const DEPLOYABLE_FILE: &'a str;

    fn get_info(&self) -> &NetContract;
    fn as_contract(&self) -> ContractInfo {
        let net = self.get_info();
        ContractInfo {
            address: Addr::unchecked(net.address.clone()),
            code_hash: net.code_hash.clone(),
        }
    }
    fn wrap_handle<Message: serde::Serialize>(
        &self,
        msg: &Message,
        sender_key: Option<&str>,
    ) -> Result<GasLog> {
        let result = test_contract_handle(
            msg,
            self.get_info(),
            sender_key.unwrap_or(Self::DEFAULT_USER),
            Some(GAS),
            Some(Self::BACKEND),
            None,
        )?
        .1;
        Ok(GasLog {
            txhash: result.txhash,
            gas_wanted: result.gas_wanted,
            gas_used: result.gas_used,
            timestamp: result.timestamp,
        })
    }
    fn wrap_init<Message: serde::Serialize>(
        msg: &Message,
        account_key: Option<&str>,
        name: Option<&str>,
    ) -> Result<NetContract> {
        test_inst_init(
            msg,
            Self::DEPLOYABLE_FILE,
            &generate_label(8),
            account_key.unwrap_or(Self::DEFAULT_USER),
            Some(STORE_GAS),
            Some(GAS),
            Some(Self::BACKEND),
            name,
        )
    }
}

mod test {
    use super::*;

    #[test]
    fn test_macro() {
        impl_deployable!(DeployableContract, "user", "test.wasm", "testnet");

        assert_eq!("user", DeployableContract::DEFAULT_USER);
        assert_eq!("test.wasm", DeployableContract::DEPLOYABLE_FILE);
        assert_eq!("testnet", DeployableContract::BACKEND);

        let test_contract = NetContract::new("test", "test", "test", "test");
        let test = DeployableContract { info: test_contract.clone() };
        assert_eq!(test.get_info(), &test_contract);
    }
}
