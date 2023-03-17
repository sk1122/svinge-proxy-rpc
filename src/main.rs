use clap::{self, Parser, Subcommand};
use svinge::{
    common::types::{Blockchain, CacheOptions},
    execution::execution::ExecutionClient,
    server::server::run_server,
};

#[derive(Subcommand, Debug)]
enum Subcommands {
    Custom {
        #[arg(short, long)]
        rpcs: Vec<String>,

        #[arg(short = 'c', long = "chain")]
        chain_id: String,

        #[arg(short = 't', long = "chain-type")]
        chain_type: Blockchain,

        #[arg(short = 'm', long = "max-connections")]
        max_connections: u64,

        #[arg(short = 'x', long = "max-responses")]
        max_responses: u64,

        #[arg(short = 'i', long = "max-retries")]
        max_retries: u64,

        #[arg(short = 'a', long = "cache-clear")]
        cache_clear: u128,

        #[arg(short = 'e', long = "exclude-methods")]
        exclude_methods: Vec<String>,
    },
    Public {
        #[arg(short = 'p', long = "with-public-providers")]
        with_public_provider: bool,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Option<Subcommands>,
}

async fn run_with_public_providers() {
    let _pol_client = ExecutionClient::new(
        Blockchain::Evm,
        "80001".into(),
        vec![
            "https://rpc.ankr.com/polygon_mumbai/".into(),
            "https://polygon-mumbai.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/".into(),
        ],
        5,
        5,
        3,
        CacheOptions {
            cache_clear: 0,
            exclude_methods: vec![],
        },
        false,
    )
    .await
    .unwrap();

    let _eth_client = ExecutionClient::new(
        Blockchain::Evm,
        "5".into(),
        vec![
            "https://rpc.ankr.com/eth_goerli/".into(),
            "https://eth-goerli.g.alchemy.com/v2/Tv9MYE2mD4zn3ziBLd6S94HvLLjTocju/".into(),
        ],
        1,
        1,
        3,
        CacheOptions {
            cache_clear: 0,
            exclude_methods: vec![],
        },
        false,
    )
    .await
    .unwrap();
}

#[tokio::main]
async fn main() {
    env_logger::init();

    unsafe { std::fs::create_dir("/tmp/svinge").unwrap_err_unchecked(); };

    let args = Args::parse();

    match args.command {
        Some(Subcommands::Custom {
            rpcs,
            chain_id,
            chain_type,
            max_connections,
            max_responses,
            max_retries,
            cache_clear,
            exclude_methods,
        }) => {
            let _client = ExecutionClient::new(
                chain_type,
                chain_id,
                rpcs,
                max_connections,
                max_responses,
                max_retries,
                CacheOptions {
                    cache_clear: cache_clear,
                    exclude_methods: exclude_methods,
                },
                false,
            )
            .await
            .unwrap();

            run_server().await.unwrap();
        }
        Some(Subcommands::Public {
            with_public_provider,
        }) => {
            if with_public_provider {
                run_with_public_providers().await;

                run_server().await.unwrap();
            }
        }
        None => println!("default"),
    }
}
