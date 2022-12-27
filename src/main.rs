use crate::rpc::{RpcRequest, IConfig, CacheOptions};


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let req= IConfig::new(vec!["https://rpc.ankr.com/polygon_mumbai".into(), "https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju".into(), "https://polygon-mumbai.infura.io/v3/a618bb907c2f4670a721be9cd51f388e".into()], 1, 1, 1, CacheOptions { exclude_methods: vec![], cache_clear: 0 }).await;

    println!("{:?}", req);
}

pub mod rpc;
mod load_balancer;
pub mod helper;