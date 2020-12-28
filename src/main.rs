#[macro_use]
extern crate prometheus;

mod config;
mod tado;

use env_logger::{Builder as LoggerBuilder, Env};
use log::{info, error};
use tokio;
use std::convert::Infallible;
use std::time::Duration;
use hyper::{service::make_service_fn, service::service_fn, Server};
use ticker::Ticker;

use config::loader as config_loader;
use tado::metrics;
use tado::client::Client as TadoClient;

#[tokio::main]
async fn main() {
    LoggerBuilder::from_env(Env::default().default_filter_or("info")).init();

    let config = config_loader::load();

    // start ticker
    run_ticker(config);

    // set up http server
    let addr = ([0, 0, 0, 0], 9898).into();
    info!("starting tado° exporter on address: {:?}", addr);

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(metrics::renderer))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // start HTTP server
    if let Err(e) = server.await {
        error!("a server error occurred: {}", e);
    }
}

fn run_ticker(config: config_loader::Config) {
    tokio::spawn(async move {
        let mut tado_client = TadoClient::new(config.username, config.password, config.client_secret);

        info!("waiting for the first tick in {} seconds...", config.ticker);

        let ticker = Ticker::new(0.., Duration::from_secs(config.ticker));
        for _ in ticker {
            metrics::set(
                tado_client.retrieve().await
            );
        }
    });
}
