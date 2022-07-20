use clap::Parser;
use std::net::SocketAddr;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Parser)]
#[clap(
    name = "Otterscan",
    about = "Local, fast and privacy-friendly block explorer."
)]
pub struct Opt {
    #[clap(long, default_value = "127.0.0.1:3000")]
    pub listen_address: SocketAddr,

    #[clap(long, default_value = "http://localhost:8545")]
    pub rpc_url: String,
}

#[tokio::main]
async fn main() {
    let opt: Opt = Opt::parse();

    let no_color = std::env::var("RUST_LOG_STYLE")
        .map(|val| val == "never")
        .unwrap_or(false);

    // tracing setup
    let env_filter = if std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_default()
        .is_empty()
    {
        EnvFilter::new("otterscan=info")
    } else {
        EnvFilter::from_default_env()
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(!no_color)
                .with_target(false),
        )
        .with(env_filter)
        .init();

    otterscan::run(opt.listen_address, opt.rpc_url)
        .await
        .unwrap()
}
