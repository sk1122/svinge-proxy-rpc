use std::{collections::HashMap, time::SystemTime};

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone)]
pub struct RPC {
    pub url: String,
    pub avg_response_time: u128,
    pub connections: u64,
    pub weight: u64,
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

#[derive(Debug)]
pub struct IConfig {
    pub rpc_urls: Vec<RPC>,
    pub max_connections: u64,
    pub max_responses: u64,
    pub max_retries: u64,
    pub cache: CacheOptions,
    pub response_results: HashMap<String, Response>
}

#[derive(Debug)]
pub struct CacheOptions {
    pub cache_clear: u128,
    pub exclude_methods: Vec<String>
}
