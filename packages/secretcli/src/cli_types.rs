use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxResponse {
    pub height: String,
    pub txhash: String,
    pub codespace: String,
    pub code: Option<u128>,
    pub data: String,
    pub raw_log: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxCompute {
    pub answers: Vec<TxAnswer>,
    pub output_logs: Vec<TxOutputLog>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxAnswer {
    pub r#type: String,
    pub input: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxOutputLog {
    pub r#type: String,
    pub attributes: Vec<TxAttribute>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxQuery {
    pub height: String,
    pub txhash: String,
    pub data: String,
    pub raw_log: String,
    pub logs: Vec<TxQueryLogs>,
    pub gas_wanted: String,
    pub gas_used: String,
    //pub tx: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxQueryLogs {
    pub msg_index: i128,
    pub log: String,
    pub events: Vec<TxQueryEvents>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxQueryEvents {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub attributes: Vec<TxQueryKeyValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TxQueryKeyValue {
    #[serde(rename = "key")]
    pub msg_key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListCodeResponse {
    pub code_id: u128,
    pub creator: String,
    pub code_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListContractCode {
    pub code_id: u128,
    pub creator: String,
    pub label: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct NetContract {
    pub label: String,
    pub id: String,
    pub address: String,
    pub code_hash: String,
}

impl NetContract {
    pub fn new(
        label: impl Into<String>,
        id: impl Into<String>,
        address: impl Into<String>,
        code_hash: impl Into<String>,
    ) -> Self {
        NetContract {
            label: label.into(),
            id: id.into(),
            address: address.into(),
            code_hash: code_hash.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GasLog {
    pub txhash: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub timestamp: String,
}

pub trait Contractable {
    fn get_contract(&self) -> (String, String);
}

impl Contractable for NetContract {
    fn get_contract(&self) -> (String, String) {
        (self.address.clone(), self.code_hash.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignedTx {
    pub pub_key: PubKey,
    pub signature: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PubKey {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredContract {
    pub id: String,
    pub code_hash: String,
}
