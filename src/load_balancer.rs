use crate::{rpc::{*}, helper};

impl IConfig {
    pub async fn new(rpc_urls: Vec<String>, max_connections: u64, max_responses: u64, max_retries: u64, cache: CacheOptions) -> IConfig {
        let mut rpcs: Vec<RPC> = vec![];

        for rpc in rpc_urls.iter() {
            let req = helper::request_and_record(rpc, &RpcRequest { jsonrpc: "2.0".into(), method: "eth_chainId".into(), params: vec![], id: "1".into() }).await.unwrap();
            rpcs.push(RPC {
                url: rpc.clone(),
                avg_response_time: req.time_taken,
                connections: 0,
                weight: 0,
                responses: vec![req]
            })
        }
        
        IConfig { rpc_urls: rpcs, max_connections, max_responses, max_retries, cache }
    }

    // pub fn sort_rpcs(self) -> IConfig {

    // }

    // pub fn request(self, request: RpcRequest) -> RpcResponse {

    // }
}