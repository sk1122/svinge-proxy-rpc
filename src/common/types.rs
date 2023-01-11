use std::time::SystemTime;
use serde_json::{Value, Map};
use serde::{Serialize, Deserialize};

#[derive(clap::ValueEnum, Debug, Clone, Serialize, Deserialize)]
pub enum Blockchain {
    Ethereum,
    Evm,
    Solana
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RPC {
    pub url: String,
    pub avg_response_time: u128,
    pub connections: u64,
    pub weight: u64,
    pub response_counter: u64,
    pub responses: Vec<Response>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub method: String,
    pub params: Vec<InnerData>,
    pub result: ResponseInnerData,
    pub time_taken: u128,
    pub start_time: SystemTime
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseInnerData {
    Text(String),
    TextArray(Vec<String>),
    NumberArray(Vec<u64>),
    Boolean(bool),
    Object(Map<String, Value>)
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: ResponseInnerData,
    pub id: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InnerData {
    Text(String),
    TextArray(Vec<String>),
    NumberArray(Vec<u64>),
    Boolean(bool)
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<InnerData>,
    pub id: String
}

#[derive(Debug)]
pub struct RpcError {
    pub error: String,
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<InnerData>,
    pub id: String,
    pub time_taken: u128
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheOptions {
    pub cache_clear: u128,
    pub exclude_methods: Vec<String>
}

impl Default for Response {
    fn default() -> Response {
        Response {
            method: "".into(),
            params: vec![],
            result: ResponseInnerData::Text("".into()),
            time_taken: 12,
            start_time: SystemTime::now()   
        }
    }
}

impl Default for RpcResponse {
    fn default() -> RpcResponse {
        RpcResponse { jsonrpc: "".into(), result: ResponseInnerData::Text("".into()), id: "".into() }
    }
}


impl Default for RpcError {
    fn default() -> RpcError {
        RpcError { jsonrpc: "".into(), error: "".into(), method:"".into(), params: vec![], time_taken: 0, id: "".into() }
    }
}