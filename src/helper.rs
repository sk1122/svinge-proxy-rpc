use crate::rpc::{RpcResponse, RpcRequest, Response, RpcError};
use reqwest::*;
use std::result::Result;
use std::time::*;

// use hyper::{Client, Request, Method, StatusCode, Body};
// use hyper::net::HttpsConnector;
// use hyper_native_tls::NativeTlsClient;

pub async fn request_and_record(url: &String, body: &RpcRequest) -> Result<Response, RpcError> {
    // println!("STARTED {} {:?}", url, body);
    let client = Client::builder().build().unwrap();
    let start = Instant::now();
    
    let res= client.post(url).json(&body).send().await.unwrap();
    // println!("completed response");
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

// pub async fn request_and_record(url: &String, body: &RpcRequest) -> Result<Response, RpcError> {
//     let ssl = NativeTlsClient::new().unwrap();
//     let connector = HttpsConnector::new(ssl);

//     let client = Client::new();
//     let start = Instant::now();
//     println!("{}", url);
//     let request = Request::builder()
//         .method(Method::POST)
//         .uri(url)
//         .header("content-type", "application/json")
//         .body(Body::from(serde_json::to_string_pretty(body).unwrap())).unwrap();

//     let res = client.request(request).await.unwrap();

//     if(res.status() == StatusCode::OK || res.status() == StatusCode::ACCEPTED || res.status() == StatusCode::CREATED) {
//         let elapsed_time = start.elapsed();
//         println!("{:?}", res.body());
//         // let body_stream = hyper::body::to_bytes(&res.body()).await.unwrap();

//         // let response: RpcResponse = serde_json::from_slice(&body_stream).unwrap();

//         let response: Response = Response {
//             method: body.method.clone(),
//             params: body.params.clone(),
//             result: String::from(""),
//             time_taken: elapsed_time.as_millis(),
//             start_time: SystemTime::now()
//         };

//         return Ok(response);
//     } else {
//         let elapsed_time = start.elapsed();
        
//         Err(RpcError {
//             jsonrpc: body.jsonrpc.clone(),
//             id: body.id.clone(),
//             error: String::from(""),
//             method: body.method.clone(),
//             params: body.params.clone(),
//             time_taken: elapsed_time.as_millis()
//         })
//     }
// }