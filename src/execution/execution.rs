use std::{collections::HashMap, time::SystemTime};
use futures::{FutureExt, future::join_all};
use serde::{Deserialize, Serialize};
use log::{info, warn};

use crate::{common::{types::*, helper::*}, extract_enum_value};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionClient {
    pub chain_type: Blockchain,
    pub chain_id: String,
    pub rpc_urls: Vec<RPC>,
    pub max_connections: u64,
    pub max_responses: u64,
    pub max_retries: u64,
    pub cache: CacheOptions,
    pub response_results: HashMap<String, Response>
}

impl ExecutionClient {
    pub async fn new(
        chain_type: Blockchain,
        chain_id: String, 
        rpc_urls: Vec<String>, 
        max_connections: u64, 
        max_responses: u64, 
        max_retries: u64, 
        cache: CacheOptions, 
        use_cached: bool
    ) -> Result<ExecutionClient, RpcError> {
        if use_cached {
            info!("Looking if there is a cached file already...");
            let text_result = std::fs::read_to_string(format!("/tmp/svinge/{}.json", chain_id.as_str()));

            match text_result {
                Ok(text) => {
                    if text.len() > 0 {
                        println!("{}", text);
                        let config = serde_json::from_str::<ExecutionClient>(&text).unwrap();
                        // println!("{:?}", config);
                        return Ok(config);
                    }
                },
                Err(_) => warn!("Not found a cached file")
            }

        }
        
        let mut rpcs: Vec<RPC> = vec![];
        let mut response_results: HashMap<String, Response> = HashMap::new();

        let demo = &RpcRequest { jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: NumberString::Number(1) };

        let mut responses = vec![];

        info!("Requesting {} RPCs and recording their performances", rpc_urls.len());

        for i in 0..5 {
            responses.append(&mut rpc_urls.iter().map(|rpc| {
                info!("Requesting {} RPC for ${} time", rpc, i);
                request_and_record(rpc, demo).boxed()
            }).collect::<Vec<_>>());
        }

        let results = join_all(responses).await;

        for (rpc, result) in rpc_urls.iter().zip(results) {
            let res = result.unwrap();

            let chain_id_en = &res.result;

            let chain_id_enum = extract_enum_value!(chain_id_en, Some(ResponseInnerData::Text(chain_id_en)) => chain_id_en);

            let chain_id_from_hex = u64::from_str_radix(&chain_id_enum[2..], 16).unwrap().to_string();

            if chain_id != chain_id_from_hex {
                warn!("{} is not equal to RPCs response {}", chain_id, chain_id_from_hex);
                return Err(RpcError { error: format!("{} is not equal to RPCs response {}", chain_id, chain_id_from_hex), jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: NumberString::Text("1".into()), time_taken: 0 })
            }

            info!("All RPCs are of chain {}", chain_id_from_hex);
            
            let avg_time_taken = res.time_taken;
            
            response_results.insert("eth_chainId".into(), res.clone());

            rpcs.push(RPC {
                url: rpc.clone(),
                avg_response_time: avg_time_taken,
                connections: 0,
                weight: 0,
                response_counter: 0,
                responses: vec![res]
            });
        }
        
        let mut new_config = ExecutionClient { 
            chain_type, 
            chain_id, 
            rpc_urls: rpcs, 
            max_connections, 
            max_responses, 
            max_retries, 
            cache, 
            response_results 
        };

        new_config.sort_rpcs();

        new_config.update_db();

        Ok(new_config)
    }

    fn update_db(&self) {
        let path = format!("/tmp/svinge/{}.json", self.chain_id.as_str());

        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }

    fn swap_rpcs(&mut self, idx: usize) {
        info!("Swapping RPCs");
        
        if idx > 0 {
            let len = self.rpc_urls.len() - 1;
            self.rpc_urls[0].response_counter = 0;
    
            //println!("{} {} {:?}", self.rpc_urls[0].url, self.rpc_urls[len].url, self.rpc_urls);
            self.rpc_urls.swap(0, len);
            //println!("{} {}", self.rpc_urls[0].url, self.rpc_urls[len].url);
            if len < idx {
                self.rpc_urls.swap(0, idx);
                //println!("{} {}", self.rpc_urls[0].url, self.rpc_urls[idx].url);
            }
        }

    }

    pub fn sort_rpcs(&mut self) {
        info!("Sorting RPCs");
        let list = &mut self.rpc_urls;

        list.sort_by(|a, b| a.avg_response_time.partial_cmp(&b.avg_response_time).unwrap());
        
        self.rpc_urls = list.to_vec();

        info!("Sorted RPCs, first -> {}, last -> {}", self.rpc_urls[0].url, self.rpc_urls.last().unwrap().url);
        
        self.update_db();
    }

    pub async fn request(&mut self, request: RpcRequest) -> Result<RpcResponse, RpcError> {
        info!("Received a request -> {:?}", request);
        if (self.rpc_urls[0].connections > self.max_connections) || (self.rpc_urls[0].response_counter > self.max_responses) {
            println!("{} {} {} {}", self.rpc_urls[0].connections, self.rpc_urls[0].response_counter, self.max_connections, self.max_responses);
            self.swap_rpcs(1);
        }
        
        self.rpc_urls[0].connections = self.rpc_urls[0].connections + 1;        
        self.update_db();
        
        let cached_result_exists = self.response_results.contains_key(&request.method);
        // println!("{}", cached_result_exists);
        if cached_result_exists {
            let cached_result = self.response_results[&request.method].clone();
            // println!("{:?} {} ", cached_result, SystemTime::now().duration_since(cached_result.start_time).unwrap().as_micros() <= self.cache.cache_clear);
            
            if SystemTime::now().duration_since(cached_result.start_time).unwrap().as_micros() <= self.cache.cache_clear {
                println!("cached {}", self.rpc_urls[0].url);
                return Ok(RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: cached_result.result, error: cached_result.error });
            }
        }

        info!("not cached {}", self.rpc_urls[0].url);

        let mut res = Response::default();

        let mut err = RpcError::default();

        for _ in 0..self.max_retries {
            let resx = request_and_record(&self.rpc_urls[0].url, &request).await;

            match resx {
                Ok(data) => {
                    res = data;
                    break;
                },
                Err(error) => err = error
            }
        }

        if res.method == Response::default().method {
            self.swap_rpcs(1);
            return Err(err);
        }

        self.rpc_urls[0].connections = self.rpc_urls[0].connections - 1;

        let cloned_res = res.clone();

        self.rpc_urls[0].responses.push(cloned_res.clone());
        self.rpc_urls[0].response_counter = self.rpc_urls[0].response_counter + 1;
        self.response_results.insert(request.method, cloned_res);
        self.rpc_urls[0].avg_response_time = (self.rpc_urls[0].avg_response_time + res.time_taken) / self.rpc_urls[0].responses.len() as u128;

        self.sort_rpcs();

        Ok(RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: res.result, error: res.error })
    }

    pub async fn request_and_validate(&mut self, request: &RpcRequest) -> Result<RpcResponse, RpcError> {
        println!("started 123");
        // let mut responses = vec![];

        let requests = self.rpc_urls.iter().map(|rpc| {
            request_and_record(&rpc.url, request).boxed()
        }).collect::<Vec<_>>();

        let results = join_all(requests).await;

        let mut response = RpcResponse::default();

        for (_, result) in self.rpc_urls.iter().zip(results) {
            let res = result.unwrap();

            response = RpcResponse {
                jsonrpc: request.jsonrpc.clone(),
                id: request.id.clone(),
                result: res.result,
                error: res.error
            }

            // todo: validate
        }

        Ok(response)
    }
}