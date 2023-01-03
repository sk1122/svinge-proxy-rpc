use clap::{self, Parser};
use svinge::{execution::execution::ExecutionClient, common::types::{Blockchain, CacheOptions}, server::server::run_server};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rpcs: Vec<String>,

    #[arg(short = 'c', long = "chain")]
    chain_id: String,
    
    #[arg(short = 't', long = "chain-type")]
    chain_type: Blockchain,

    #[arg(short = 'm', long = "max-connections")]
    pub max_connections: u64,

    #[arg(short = 'x', long = "max-responses")]
    pub max_responses: u64,

    #[arg(short = 'i', long = "max-retries")]
    pub max_retries: u64,

    #[arg(short = 'a', long = "cache-clear")]
    pub cache_clear: u128,

    #[arg(short = 'e', long = "exclude-methods")]
    exclude_methods: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    let _client = ExecutionClient::new(
        args.chain_type,
        args.chain_id,
        args.rpcs,
        args.max_connections,
        args.max_responses,
        args.max_retries,
        CacheOptions {
            cache_clear: args.cache_clear,
            exclude_methods: args.exclude_methods
        },
        false
    ).await.unwrap();

    run_server().await.unwrap();
}