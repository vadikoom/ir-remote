//#![deny(warnings)]
extern crate core;

use std::net::Ipv4Addr;
use std::process;
use tokio::select;
use tracing::{error, info, info_span};

mod bot;
mod config;
mod server;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    let _ = info_span!("main").entered();

    let bind_address = std::env::var("IR_LISTEN_IP")
        .expect("env variable IR_LISTEN_IP should be defined")
        .parse::<Ipv4Addr>()
        .expect("BIND_ADDRESS should be a valid Ipv4Addr");

    let bind_port = std::env::var("IR_LISTEN_PORT")
        .expect("env variable IR_LISTEN_PORT should be defined")
        .parse::<u16>()
        .expect("BIND_PORT should be a valid u16 number");

    info!("allowed users: {:?}", config::get_users());

    let cfg = config::get_config();

    let (mut runner, handle) = server::new(bind_address, bind_port).await.unwrap();

    info!("Starting bot with default configuration");
    select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Got ctrl_c signal, exiting gracefully");
        }

        res = runner.run() => {
            error!("Server unexpectedly terminated: {:?}", res);
            process::exit(1);
        }

        res = bot::run(cfg, handle) => {
            error!("Bot unexpectedly terminated: {:?}", res);
            process::exit(1);
        }
    }
}
