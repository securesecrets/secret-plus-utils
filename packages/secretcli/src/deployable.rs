use crate::{
    cli_types::{NetContract},
};
use cosmwasm_std::{ContractInfo, Addr};

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

        impl Deployable for $x {
            const DEFAULT_USER: &'static str = $user;
            const BACKEND: &'static str = $backend;
            const DEPLOYABLE_FILE: &'static str = $deployable_file;

            fn get_info(&self) -> &NetContract {
                &self.info
            }
            fn set_info(&mut self, info: &NetContract) { self.info = info.clone() }
        }
    };
}

pub trait Deployable {
    const DEFAULT_USER: &'static str;
    const BACKEND: &'static str;
    const DEPLOYABLE_FILE: &'static str;

    fn default_user(&self) -> &str { Self::DEFAULT_USER }
    fn backend(&self) -> &str { Self::BACKEND }
    fn file(&self) -> &str { Self::DEPLOYABLE_FILE }
    fn get_info(&self) -> &NetContract;
    fn as_contract(&self) -> ContractInfo {
        let net = self.get_info();
        ContractInfo {
            address: Addr::unchecked(net.address.clone()),
            code_hash: net.code_hash.clone(),
        }
    }
    fn set_info(&mut self, info: &NetContract);
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
