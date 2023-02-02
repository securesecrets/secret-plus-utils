//! Multitest is a design to simulate a blockchain environment in pure Rust.
//! This allows us to run unit tests that involve contract -> contract,
//! and contract -> bank interactions. This is not intended to be a full blockchain app
//! but to simulate the Cosmos SDK x/wasm module close enough to gain confidence in
//! multi-contract deployements before testing them on a live blockchain.
//!
//! To understand the design of this module, please refer to `../DESIGN.md`

pub(crate) mod app;
pub(crate) mod bank;
#[allow(clippy::type_complexity)]
pub(crate) mod contracts;
pub mod custom_handler;
pub mod error;
pub(crate) mod executor;
pub(crate) mod gov;
pub(crate) mod ibc;
pub(crate) mod module;
pub(crate) mod prefixed_storage;
pub(crate) mod staking;
pub(crate) mod test_helpers;
pub(crate) mod transactions;
pub(crate) mod wasm;

pub use crate::app::{
    custom_app, next_block, App, AppBuilder, BasicApp, BasicAppBuilder, CosmosRouter, Router,
    SudoMsg,
};
pub use crate::bank::{Bank, BankKeeper, BankSudo};
pub use crate::contracts::{Contract, ContractInstantiationInfo, ContractWrapper};
pub use crate::executor::{AppResponse, Executor};
pub use crate::ibc::Ibc;
pub use crate::module::{FailingModule, Module};
pub use crate::staking::{Distribution, DistributionKeeper, Staking, StakingKeeper, StakingSudo};
pub use crate::wasm::{Wasm, WasmKeeper, WasmSudo};
pub use nanoid;
