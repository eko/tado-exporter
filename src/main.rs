#[macro_use]
extern crate prometheus;

mod config;
mod tado;

use std::convert::Infallible;
use hyper::{service::make_service_fn, service::service_fn, Server};

use config::loader as config_loader;
use tado::metrics::renderer;
use tado::client::Client as TadoClient;

#[tokio::main]
async fn main() {
    let config = config_loader::load();

    let tadoClient = TadoClient{
        httpClient: reqwest::Client::new(),
        username: config.username,
        password: config.password,
    };

    tadoClient.authenticate();

    let addr = ([0, 0, 0, 0], 9898).into();
    println!("starting tado° exporter on address: {:?}", addr);

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(renderer))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("a server error occured: {}", e);
    }
}
