use cosmwasm_std::StdResult;
use serde::{de::DeserializeOwned, Serialize};

mod bincode2;
mod json;

/// This trait represents the ability to both serialize and deserialize using a specific format.
///
/// This is useful for types that want to have a default mode of serialization, but want
/// to allow users to override it if they want to.
///
/// It is intentionally simple at the moment to keep the implementation easy.
pub trait Serde {
    fn serialize<T: Serialize>(obj: &T) -> StdResult<Vec<u8>>;
    fn deserialize<T: DeserializeOwned>(data: &[u8]) -> StdResult<T>;
}

pub use self::bincode2::Bincode2;
pub use self::json::Json;
