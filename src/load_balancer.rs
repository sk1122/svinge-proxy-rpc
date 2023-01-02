use std::{collections::HashMap, time::SystemTime};

use crate::{rpc::{*}, helper};

impl IConfig {
    pub async fn new(rpc_urls: Vec<String>, max_connections: u64, max_responses: u64, max_retries: u64, cache: CacheOptions, use_cached: bool) -> IConfig {
        if use_cached {
            let text = std::fs::read_to_string("./a.json").unwrap();

            if text.len() > 0 {
                let config = serde_json::from_str::<IConfig>(&text).unwrap();
                // println!("{:?}", config);
                return config;
            }
        }
        
        let mut rpcs: Vec<RPC> = vec![];
        let mut response_results: HashMap<String, Response> = HashMap::new();

        for rpc in rpc_urls.iter() {
            for _ in 0..5 {
                let req = helper::request_and_record(rpc, &RpcRequest { jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: "1".into() }).await.unwrap();
                rpcs.push(RPC {
                    url: rpc.clone(),
                    avg_response_time: req.time_taken,
                    connections: 0,
                    weight: 0,
                    responses: vec![req.clone()]
                });
    
                response_results.insert("eth_chainId".into(), req);
            }
        }
        
        let mut new_config = IConfig { rpc_urls: rpcs, max_connections, max_responses, max_retries, cache, response_results };

        new_config.sort_rpcs();

        new_config.update_db();

        new_config
    }

    fn update_db(&self) {
        let path = "./a.json";

        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }

    fn swap_rpcs(&mut self, idx: usize) {
        let len = self.rpc_urls.len() - 1;

        self.rpc_urls.swap(0, len);
        self.rpc_urls.swap(0, idx);
    }

    pub fn sort_rpcs(&mut self) {
        let list = &mut self.rpc_urls;

        list.sort_by(|a, b| a.avg_response_time.partial_cmp(&b.avg_response_time).unwrap());
        
        self.rpc_urls = list.to_vec();
        
        self.update_db();
    }

    pub async fn request(&mut self, request: RpcRequest) -> RpcResponse {
        if self.rpc_urls[0].connections > self.max_connections {
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
                // println!("cached");
                return RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: cached_result.result };
            }
        }

        // println!("not cached");

        let res = helper::request_and_record(&self.rpc_urls[0].url, &request).await.unwrap();

        self.rpc_urls[0].connections = self.rpc_urls[0].connections - 1;

        let cloned_res = res.clone();

        self.rpc_urls[0].responses.push(cloned_res.clone());
        self.response_results.insert(request.method, cloned_res);
        self.rpc_urls[0].avg_response_time = (self.rpc_urls[0].avg_response_time + res.time_taken) / self.rpc_urls[0].responses.len() as u128;

        self.sort_rpcs();

        return RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: res.result };
    }
}