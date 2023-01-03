use crate::common::types::{RpcResponse, RpcRequest, Response, RpcError};
use reqwest::*;
use std::result::Result;
use std::time::*;


pub async fn request_and_record(url: &String, body: &RpcRequest) -> Result<Response, RpcError> {
    // println!("STARTED {} {:?}", url, body);
    let client = Client::builder().build().unwrap();
    let start = Instant::now();
    
    let res= client.post(url).json(&body).send().await.unwrap();
    // println!("completed response");
    let status = res.status();
    println!("{:?}", status);
    if status == StatusCode::OK || status == StatusCode::ACCEPTED || status == StatusCode::CREATED {
        let response_result = res.json::<RpcResponse>().await;
        let elapsed_time = start.elapsed();
    
        let mut response = RpcResponse::default();

        match response_result {
            Ok(res) => response = res,
            Err(err) => return Err(RpcError { error: err.to_string(), jsonrpc: body.jsonrpc.clone(), method: body.method.clone(), params: body.params.clone(), id: body.id.clone(), time_taken: elapsed_time.as_millis() })
        }
    
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