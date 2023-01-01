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

    pub fn sort_rpcs(&mut self) {
        let list = &mut self.rpc_urls;

        list.sort_by(|a, b| a.avg_response_time.partial_cmp(&b.avg_response_time).unwrap());

        self.rpc_urls = list.to_vec();
    }

    pub async fn request(self, request: RpcRequest) -> RpcResponse {
        let res = helper::request_and_record(&self.rpc_urls[0].url, &request).await.unwrap();

        return RpcResponse { jsonrpc: request.jsonrpc, id: request.id, result: res.result };
    }
}