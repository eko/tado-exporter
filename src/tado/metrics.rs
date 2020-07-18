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

    pub static ref ACTIVITY_AC_POWER: GaugeVec = register_gauge_vec!(
        "tado_activity_ac_power_value",
        "The value of ac power in a specific zone.",
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
        // The setting temperature may be null in the API response, if the
        // zone's heating mode is turned off. If the temperature setting is
        // absent, from the API response we'll simply not set its gauge values.
        if let Some(setting_temperature) = zone.state_response.setting.temperature {
            // setting temperature
            let value: f64 = setting_temperature.celsius;
            SETTING_TEMPERATURE.with_label_values(&[zone.name.as_str(), "celsius"]).set(value);
            info!("-> {} -> setting temperature (celsius): {}", zone.name, value);

            let value: f64 = setting_temperature.fahrenheit;
            SETTING_TEMPERATURE.with_label_values(&[zone.name.as_str(), "fahrenheit"]).set(value);
            info!("-> {} -> setting temperature (fahrenheit): {}", zone.name, value);
        } else {
            info!("-> {} -> setting temperature (celsius): Off", zone.name);
            info!("-> {} -> setting temperature (fahrenheit): Off", zone.name);
        }

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
        if let Some(heating_power) = zone.state_response.activityDataPoints.heatingPower {
            let value: f64 = heating_power.percentage;
            ACTIVITY_HEATING_POWER.with_label_values(&[zone.name.as_str()]).set(value);
            info!("-> {} -> heating power: {}%", zone.name, value);
        }

        // ac power
        if let Some(ac_power) = zone.state_response.activityDataPoints.acPower {
            let value: f64 = match ac_power.value.as_str() {
                "ON"  => 1.0,
                "OFF" => 0.0,
                _     => 0.0,
            };

            ACTIVITY_AC_POWER.with_label_values(&[zone.name.as_str()]).set(value);
            info!("-> {} -> ac power: {}", zone.name, value);
        }
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
