use std::{collections::HashMap, time::SystemTime};

use crate::{rpc::{*}, helper};

impl IConfig {
    pub async fn new(rpc_urls: Vec<String>, max_connections: u64, max_responses: u64, max_retries: u64, cache: CacheOptions) -> IConfig {
        let mut rpcs: Vec<RPC> = vec![];
        let mut response_results: HashMap<String, Response> = HashMap::new();

        for rpc in rpc_urls.iter() {
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
        
        let mut new_config = IConfig { rpc_urls: rpcs, max_connections, max_responses, max_retries, cache, response_results };

        new_config.sort_rpcs();

        new_config
    }

    pub fn sort_rpcs(&mut self) {
        let list = &mut self.rpc_urls;

        list.sort_by(|a, b| a.avg_response_time.partial_cmp(&b.avg_response_time).unwrap());

        self.rpc_urls = list.to_vec();
    }

    pub async fn request(&mut self, request: RpcRequest) -> RpcResponse {
        self.rpc_urls[0].connections = self.rpc_urls[0].connections + 1;
        
        let cached_result_exists = self.response_results.contains_key(&request.method);

        if cached_result_exists {
            let cached_result = self.response_results[&request.method].clone();
            println!("{}", SystemTime::now().duration_since(cached_result.start_time).unwrap().as_micros());
            if SystemTime::now().duration_since(cached_result.start_time).unwrap().as_micros() <= self.cache.cache_clear {
                println!("cached");
                return RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: cached_result.result };
            }
        }

        println!("not cached");

        let res = helper::request_and_record(&self.rpc_urls[0].url, &request).await.unwrap();

        self.rpc_urls[0].connections = self.rpc_urls[0].connections - 1;

        let cloned_res = res.clone();

        self.rpc_urls[0].responses.push(cloned_res);
        self.rpc_urls[0].avg_response_time = (self.rpc_urls[0].avg_response_time + res.time_taken) / self.rpc_urls[0].responses.len() as u128;

        self.sort_rpcs();

        return RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: res.result };
    }
}