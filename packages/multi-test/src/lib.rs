//! Multitest is a design to simulate a blockchain environment in pure Rust.
//! This allows us to run unit tests that involve contract -> contract,
//! and contract -> bank interactions. This is not intended to be a full blockchain app
//! but to simulate the Cosmos SDK x/wasm module close enough to gain confidence in
//! multi-contract deployements before testing them on a live blockchain.
//!
//! To understand the design of this module, please refer to `../DESIGN.md`

#[cfg(not(target_arch = "wasm32"))]
mod app;
#[cfg(not(target_arch = "wasm32"))]
mod bank;
#[allow(clippy::type_complexity)]
mod contracts;
#[cfg(not(target_arch = "wasm32"))]
pub mod custom_handler;
pub mod error;
mod executor;
#[cfg(not(target_arch = "wasm32"))]
mod module;
#[cfg(not(target_arch = "wasm32"))]
mod staking;
mod test_helpers;
mod transactions;
#[cfg(not(target_arch = "wasm32"))]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::app::{
    custom_app, next_block, App, AppBuilder, BasicApp, BasicAppBuilder, CosmosRouter, Router,
    SudoMsg,
};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::bank::{Bank, BankKeeper, BankSudo};
pub use crate::contracts::{Contract, ContractWrapper};
pub use crate::executor::{AppResponse, Executor};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::module::Module;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::staking::{FailingDistribution, FailingStaking, Staking, StakingSudo};
#[cfg(not(target_arch = "wasm32"))]
pub use crate::wasm::{Wasm, WasmKeeper, WasmSudo};
pub use nanoid;
