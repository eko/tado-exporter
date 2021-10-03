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
        &["zone", "type"]
    ).unwrap();

    pub static ref ACTIVITY_AC_POWER: GaugeVec = register_gauge_vec!(
        "tado_activity_ac_power_value",
        "The value of ac power in a specific zone.",
        &["zone", "type"]
    ).unwrap();

    pub static ref SETTING_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "tado_setting_temperature_value",
        "The temperature of a specific zone in celsius degres.",
        &["zone", "type", "unit"]
    ).unwrap();

    pub static ref SENSOR_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "tado_sensor_temperature_value",
        "The temperature of a specific zone in celsius degres.",
        &["zone", "type", "unit"]
    ).unwrap();

    pub static ref SENSOR_HUMIDITY_PERCENTAGE: GaugeVec = register_gauge_vec!(
        "tado_sensor_humidity_percentage",
        "The % of humidity in a specific zone.",
        &["zone", "type"]
    ).unwrap();
}

pub fn set(zones: Vec<ZoneStateResponse>) {
    for zone in zones {
        let device_type: String = zone.state_response.setting.deviceType;

        // The setting temperature may be null in the API response, if the
        // zone's heating mode is turned off. If the temperature setting is
        // absent, from the API response we'll simply not set its gauge values.
        if let Some(setting_temperature) = zone.state_response.setting.temperature {
            // setting temperature
            let value: f64 = setting_temperature.celsius;
            SETTING_TEMPERATURE.with_label_values(&[zone.name.as_str(), device_type.as_str(), "celsius"]).set(value);
            info!("-> {} ({}) -> setting temperature (celsius): {}", zone.name, device_type.as_str(), value);

            let value: f64 = setting_temperature.fahrenheit;
            SETTING_TEMPERATURE.with_label_values(&[zone.name.as_str(), device_type.as_str(), "fahrenheit"]).set(value);
            info!("-> {} ({}) -> setting temperature (fahrenheit): {}", zone.name, device_type.as_str(), value);
        } else {
            info!("-> {} ({}) -> setting temperature (celsius): Off", zone.name, device_type.as_str());
            info!("-> {} ({}) -> setting temperature (fahrenheit): Off", zone.name, device_type.as_str());
        }

        // sensor temperature
        if let Some(inside_temperature) = zone.state_response.sensorDataPoints.insideTemperature {
            // celsius
            let value: f64 = inside_temperature.celsius;
            SENSOR_TEMPERATURE.with_label_values(&[zone.name.as_str(), device_type.as_str(), "celsius"]).set(value);
            info!("-> {} ({}) -> sensor temperature (celsius): {}", zone.name, device_type.as_str(), value);

            // fahrenheit
            let value: f64 = inside_temperature.fahrenheit;
            SENSOR_TEMPERATURE.with_label_values(&[zone.name.as_str(), device_type.as_str(), "fahrenheit"]).set(value);
            info!("-> {} ({}) -> sensor temperature (fahrenheit): {}", zone.name, device_type.as_str(), value);
        }

        // sensor humidity
        if let Some(humidity) = zone.state_response.sensorDataPoints.humidity {
            let value: f64 = humidity.percentage;
            SENSOR_HUMIDITY_PERCENTAGE.with_label_values(&[zone.name.as_str(), device_type.as_str()]).set(value);
            info!("-> {} ({}) -> sensor humidity: {}%", zone.name, device_type.as_str(), value);
        }

        // heating power
        if let Some(heating_power) = zone.state_response.activityDataPoints.heatingPower {
            let value: f64 = heating_power.percentage;
            ACTIVITY_HEATING_POWER.with_label_values(&[zone.name.as_str(), device_type.as_str()]).set(value);
            info!("-> {} ({}) -> heating power: {}%", zone.name, device_type.as_str(), value);
        }

        // ac power
        if let Some(ac_power) = zone.state_response.activityDataPoints.acPower {
            let value: f64 = match ac_power.value.as_str() {
                "ON"  => 1.0,
                "OFF" => 0.0,
                _     => 0.0,
            };

            ACTIVITY_AC_POWER.with_label_values(&[zone.name.as_str(), device_type.as_str()]).set(value);
            info!("-> {} ({}) -> ac power: {}", zone.name, device_type.as_str(), value);
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
