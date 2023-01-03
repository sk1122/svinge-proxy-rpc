use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
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
    pub params: Vec<String>,
    pub result: String,
    pub time_taken: u128,
    pub start_time: SystemTime
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub result: String,
    pub id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<String>,
    pub id: String
}

#[derive(Debug)]
pub struct RpcError {
    pub error: String,
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<String>,
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
            result: "".into(),
            time_taken: 12,
            start_time: SystemTime::now()   
        }
    }
}

impl Default for RpcResponse {
    fn default() -> RpcResponse {
        RpcResponse { jsonrpc: "".into(), result: "".into(), id: "".into() }
    }
}


impl Default for RpcError {
    fn default() -> RpcError {
        RpcError { jsonrpc: "".into(), error: "".into(), method:"".into(), params: vec![], time_taken: 0, id: "".into() }
    }
}