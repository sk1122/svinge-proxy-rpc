use crate::rpc::{RpcResponse, RpcRequest, Response, RpcError};
use reqwest::*;
use std::result::Result;
use std::time::*;

pub async fn request_and_record(url: &String, body: &RpcRequest) -> Result<Response, RpcError> {
    let client = Client::new();
    let start = Instant::now();
    
    let res= client.post(url).json(&body).send().await.unwrap();
    let status = res.status();

    if status == StatusCode::OK || status == StatusCode::ACCEPTED || status == StatusCode::CREATED {
        let response = res.json::<RpcResponse>().await.unwrap();
        let elapsed_time = start.elapsed();
    
        let response: Response = Response {
            method: body.method.clone(),
            params: body.params.clone(),
            result: response.result,
            time_taken: elapsed_time.as_millis(),
            start_time: SystemTime::now()
        };
    
        Ok(response)
    } else {
        let elapsed_time = start.elapsed();
        
        Err(RpcError {
            jsonrpc: body.jsonrpc.clone(),
            id: body.id.clone(),
            error: res.text().await.unwrap().into(),
            method: body.method.clone(),
            params: body.params.clone(),
            time_taken: elapsed_time.as_millis()
        })
    }
    
}