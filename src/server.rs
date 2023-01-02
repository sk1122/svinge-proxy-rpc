use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, body::BoxBody, http::header::ContentType};

use crate::rpc::{RpcRequest, IConfig, RpcResponse, CacheOptions};

impl Responder for RpcResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

#[post("/eth")]
async fn eth(req_body: web::Json<RpcRequest>) -> impl Responder {
    let mut req= IConfig::new(vec!["https://rpc.ankr.com/polygon_mumbai/".into(), "https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/".into()], 1, 1, 1, CacheOptions { exclude_methods: vec![], cache_clear: 2000000 }, true).await;
    
    // println!("{:?}", req.rpc_urls);

    let res = req.request(req_body.into_inner()).await;

    res
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey There")
}


pub async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(eth)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}