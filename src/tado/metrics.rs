use std::convert::Infallible;

use lazy_static::lazy_static;
use hyper::{header::CONTENT_TYPE, Body, Request, Response};
use prometheus::{Encoder, Gauge, TextEncoder};

lazy_static! {
    pub static ref TEMPERATURE_GAUGE: Gauge = register_gauge!(opts!(
        "tado_temperature_degre",
        "The temperature of a piece in celsius degres.",
        labels! {"piece" => "all",}
    ))
    .unwrap();
}

pub async fn renderer(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let metrics = prometheus::gather();
    let mut buffer = vec![];

    let encoder = TextEncoder::new();
    encoder.encode(&metrics, &mut buffer).unwrap();

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    Ok(response)
}