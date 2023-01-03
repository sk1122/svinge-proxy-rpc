use actix_web::{post, web, App, HttpResponse, HttpServer, Responder, body::BoxBody, http::header::ContentType, ResponseError, http::StatusCode};
use crate::common::types::{RpcRequest, RpcResponse, CacheOptions, Blockchain};
use crate::execution::execution::ExecutionClient;
use derive_more::{Display, Error};

impl Responder for RpcResponse {
    type Body = BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[derive(Debug, Display, Error)]
#[display(fmt = "{}", error_message)]
struct ServerError {
    error_message: String,
    status: StatusCode
}

impl ResponseError for ServerError {}

#[post("/eth")]
async fn eth(req_body: web::Json<RpcRequest>) -> Result<impl Responder, ServerError> {
    let req_result= ExecutionClient::new(Blockchain::Evm, "80001".into(), vec!["https://rpc.ankr.com/polygon_mumbai/".into(), "https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/".into()], 1, 1, 5, CacheOptions { exclude_methods: vec![], cache_clear: 2000000 }, true).await;

    match req_result {
        Ok(mut req) => {
            let res = req.request(req_body.into_inner()).await;
        
            match res {
                Ok(result) => Ok(result),
                Err(err) => Err(ServerError { error_message: err.error, status: StatusCode::BAD_REQUEST })
            }
        },
        Err(err) => Err(ServerError { error_message: err.error, status: StatusCode::BAD_REQUEST })
    }

}

pub async fn run_server() -> std::io::Result<()> {
    println!("Running server on port {}🎉", 8080);
    
    HttpServer::new(|| {
        App::new()
            .service(eth)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}