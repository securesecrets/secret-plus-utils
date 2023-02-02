//! Multitest is a design to simulate a blockchain environment in pure Rust.
//! This allows us to run unit tests that involve contract -> contract,
//! and contract -> bank interactions. This is not intended to be a full blockchain app
//! but to simulate the Cosmos SDK x/wasm module close enough to gain confidence in
//! multi-contract deployements before testing them on a live blockchain.
//!
//! To understand the design of this module, please refer to `../DESIGN.md`

#[cfg(not(target_arch = "wasm32"))]
pub mod multi;
#[cfg(not(target_arch = "wasm32"))]
pub use multi::*;
