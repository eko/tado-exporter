use std::convert::Infallible;

use super::model::ZoneStateResponse;

use log::info;
use lazy_static::lazy_static;
use hyper::{header::CONTENT_TYPE, Body, Request, Response};
use prometheus::{Encoder, GaugeVec, TextEncoder};

lazy_static! {
    pub static ref ACTIVITY_HEATING_POWER: GaugeVec = register_gauge_vec!(
        "tado_activity_heating_power_percentage",
        "The % of heating power in a specific zone.",
        &["zone"]
    ).unwrap();

    pub static ref SETTING_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "tado_setting_temperature_value",
        "The temperature of a specific zone in celsius degres.",
        &["zone", "unit"]
    ).unwrap();

    pub static ref SENSOR_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "tado_sensor_temperature_value",
        "The temperature of a specific zone in celsius degres.",
        &["zone", "unit"]
    ).unwrap();

    pub static ref SENSOR_HUMIDITY_PERCENTAGE: GaugeVec = register_gauge_vec!(
        "tado_sensor_humidity_percentage",
        "The % of humidity in a specific zone.",
        &["zone"]
    ).unwrap();
}

pub fn set(zones: Vec<ZoneStateResponse>) {
    for zone in zones {
        // setting temperature
        let value: f64 = zone.state_response.setting.temperature.celsius;
        SETTING_TEMPERATURE.with_label_values(&[zone.name.as_str(), "celsius"]).set(value);
        info!("-> {} -> setting temperature (celsius): {}", zone.name, value);

        let value: f64 = zone.state_response.setting.temperature.fahrenheit;
        SETTING_TEMPERATURE.with_label_values(&[zone.name.as_str(), "fahrenheit"]).set(value);
        info!("-> {} -> setting temperature (fahrenheit): {}", zone.name, value);

        // sensor temperature
        let value: f64 = zone.state_response.sensorDataPoints.insideTemperature.celsius;
        SENSOR_TEMPERATURE.with_label_values(&[zone.name.as_str(), "celsius"]).set(value);
        info!("-> {} -> sensor temperature (celsius): {}", zone.name, value);

        let value: f64 = zone.state_response.sensorDataPoints.insideTemperature.fahrenheit;
        SENSOR_TEMPERATURE.with_label_values(&[zone.name.as_str(), "fahrenheit"]).set(value);
        info!("-> {} -> sensor temperature (fahrenheit): {}", zone.name, value);

        // sensor humidity
        let value: f64 = zone.state_response.sensorDataPoints.humidity.percentage;
        SENSOR_HUMIDITY_PERCENTAGE.with_label_values(&[zone.name.as_str()]).set(value);
        info!("-> {} -> sensor humidity: {}%", zone.name, value);

        // heating power
        let value: f64 = zone.state_response.activityDataPoints.heatingPower.percentage;
        ACTIVITY_HEATING_POWER.with_label_values(&[zone.name.as_str()]).set(value);
        info!("-> {} -> heating power: {}%", zone.name, value);
    }
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