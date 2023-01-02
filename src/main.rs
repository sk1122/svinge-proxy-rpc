use crate::rpc::{IConfig, CacheOptions, RpcRequest};
use crate::server::run_server;
use env_logger;


#[tokio::main]
// #[actix_web::main]
async fn main() {
    env_logger::init();
    // println!("Hello, world!");

    run_server().await.unwrap();
}

pub mod rpc;
mod load_balancer;
pub mod helper;
pub mod server;