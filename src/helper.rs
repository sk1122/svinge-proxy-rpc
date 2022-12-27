use crate::rpc::{RpcResponse, RpcRequest, Response};
use reqwest::*;
use std::time::Instant;

pub async fn request_and_record(url: &String, body: &RpcRequest) -> Option<Response> {
    let client = Client::new();
    let start = Instant::now();
    
    let res: RpcResponse = client.post(url).json(&body).send().await.unwrap().json::<RpcResponse>().await.unwrap();
    
    let elapsed_time = start.elapsed();

    let response: Response = Response {
        method: body.method.clone(),
        params: body.params.clone(),
        result: res.result.clone(),
        time_taken: elapsed_time.as_millis()
    };

    Some(response)
}