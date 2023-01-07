use std::{collections::HashMap, time::SystemTime};
use futures::{stream, StreamExt, FutureExt, join, future::join_all};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{common::{types::*, helper::*}};

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

const PARALLEL_REQUESTS: usize = 8;

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
            let text_result = std::fs::read_to_string(format!("./{}.json", chain_id.as_str()));

            match text_result {
                Ok(text) => {
                    if text.len() > 0 {
                        let config = serde_json::from_str::<ExecutionClient>(&text).unwrap();
                        // println!("{:?}", config);
                        return Ok(config);
                    }
                },
                Err(_) => println!("error")
            }

        }
        
        let mut rpcs: Vec<RPC> = vec![];
        let mut response_results: HashMap<String, Response> = HashMap::new();

        let demo = &RpcRequest { jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: "1".into() };

        let responses = rpc_urls.iter().map(|rpc| {
            request_and_record(rpc, demo).boxed()
        }).collect::<Vec<_>>();

        let results = join_all(responses).await;

        for (rpc, result) in rpc_urls.iter().zip(results) {
            let res = result.unwrap();

            let chain_id_from_hex = u64::from_str_radix(&res.result[2..], 16).unwrap().to_string();

            if chain_id != chain_id_from_hex {
                return Err(RpcError { error: format!("{} is not equal to RPCs response {}", chain_id, chain_id_from_hex), jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: "1".into(), time_taken: 0 })
            }
            
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

        // for rpc in rpc_urls.iter() {
        //     let mut avg_time_taken = 0;
        //     let mut requests = vec![];
            
        //     for i in 0..5 {
        //         let req = request_and_record(rpc, &RpcRequest { jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: "1".into() }).await.unwrap();
        //         println!("{}", &req.result[2..]);
        //         let chain_id_from_hex = u64::from_str_radix(&req.result[2..], 16).unwrap().to_string();

        //         if chain_id != chain_id_from_hex {
        //             return Err(RpcError { error: format!("{} is not equal to RPCs response {}", chain_id, chain_id_from_hex), jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: "1".into(), time_taken: 0 })
        //         }
                
        //         avg_time_taken = (avg_time_taken + req.time_taken) / (i + 1);
                
        //         response_results.insert("eth_chainId".into(), req.clone());
                
        //         requests.push(req);
        //     }

        //     rpcs.push(RPC {
        //         url: rpc.clone(),
        //         avg_response_time: avg_time_taken,
        //         connections: 0,
        //         weight: 0,
        //         response_counter: 0,
        //         responses: requests
        //     });
        // }
        
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
        let path = format!("./{}.json", self.chain_id.as_str());

        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }

    fn swap_rpcs(&mut self, idx: usize) {
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

    pub fn sort_rpcs(&mut self) {
        let list = &mut self.rpc_urls;

        list.sort_by(|a, b| a.avg_response_time.partial_cmp(&b.avg_response_time).unwrap());
        
        self.rpc_urls = list.to_vec();
        
        self.update_db();
    }

    pub async fn request(&mut self, request: RpcRequest) -> Result<RpcResponse, RpcError> {
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
                return Ok(RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: cached_result.result });
            }
        }

        println!("not cached {}", self.rpc_urls[0].url);

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

        Ok(RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: res.result })
    }

    pub async fn request_and_validate(&mut self, request: &RpcRequest) {
        println!("started 123");
        let urls: Vec<String> = vec![String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"), String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/"),String::from("https://rpc.ankr.com/polygon_mumbai/"), String::from("https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/")];

        // let mut responses = vec![];

        let requests = urls.iter().map(|rpc| {
            request_and_record(rpc, request).boxed()
        }).collect::<Vec<_>>();

        let results = join_all(requests).await;

        for (rpc, result) in urls.iter().zip(results) {
            let res = result.unwrap();

            println!("{:?}", res);

            // todo: validate
        }
    }
}