use std::convert::Infallible;

use super::model::{WeatherApiResponse, ZoneStateResponse};

use hyper::{header::CONTENT_TYPE, Body, Request, Response};
use lazy_static::lazy_static;
use log::info;
use prometheus::{Encoder, GaugeVec, TextEncoder};

lazy_static! {
    pub static ref ACTIVITY_HEATING_POWER: GaugeVec = register_gauge_vec!(
        "tado_activity_heating_power_percentage",
        "The % of heating power in a specific zone.",
        &["zone", "type"]
    )
    .unwrap();
    pub static ref ACTIVITY_AC_POWER: GaugeVec = register_gauge_vec!(
        "tado_activity_ac_power_value",
        "The value of ac power in a specific zone.",
        &["zone", "type"]
    )
    .unwrap();
    pub static ref SETTING_POWER: GaugeVec = register_gauge_vec!(
        "tado_setting_power_value",
        "The value of the power setting (1.0 = ON, 0.0 = OFF) in a specific zone.",
        &["zone", "type"]
    )
    .unwrap();
    pub static ref SETTING_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "tado_setting_temperature_value",
        "The temperature of a specific zone in celsius degres.",
        &["zone", "type", "unit"]
    )
    .unwrap();
    pub static ref SENSOR_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "tado_sensor_temperature_value",
        "The temperature of a specific zone in celsius degres.",
        &["zone", "type", "unit"]
    )
    .unwrap();
    pub static ref SENSOR_HUMIDITY_PERCENTAGE: GaugeVec = register_gauge_vec!(
        "tado_sensor_humidity_percentage",
        "The % of humidity in a specific zone.",
        &["zone", "type"]
    )
    .unwrap();
    pub static ref WEATHER_SOLAR_INTENSITY: GaugeVec = register_gauge_vec!(
        "weather_solar_intensity",
        "Solar intensity outside the house.",
        &[]
    )
    .unwrap();
    pub static ref WEATHER_OUTSIDE_TEMPERATURE: GaugeVec = register_gauge_vec!(
        "weather_outside_temperature",
        "Temperature outside the house.",
        &["unit"]
    )
    .unwrap();
    pub static ref SENSOR_WINDOW_OPENED: GaugeVec = register_gauge_vec!(
        "tado_sensor_window_opened",
        "1 if the sensor detected a window is open, 0 otherwise.",
        &["zone", "type"]
    )
    .unwrap();
}

pub fn set_zones(zones: Vec<ZoneStateResponse>) {
    for zone in zones {
        let device_type: String = zone.state_response.setting.deviceType;

        if let Some(power) = zone.state_response.setting.power.as_deref() {
            SETTING_POWER
                .with_label_values(&[zone.name.as_str(), device_type.as_str()])
                .set(if power == "ON" { 1.0 } else { 0.0 });
            info!(
                "-> {} ({}) -> setting power: {}",
                zone.name,
                device_type.as_str(),
                power
            );
        }

        // The setting temperature may be null in the API response, if the
        // zone's heating mode is turned off. Its series are then removed:
        // keeping them would expose the last temperature set as still active.
        if let Some(setting_temperature) = zone.state_response.setting.temperature {
            // setting temperature
            let value: f64 = setting_temperature.celsius;
            SETTING_TEMPERATURE
                .with_label_values(&[zone.name.as_str(), device_type.as_str(), "celsius"])
                .set(value);
            info!(
                "-> {} ({}) -> setting temperature (celsius): {}",
                zone.name,
                device_type.as_str(),
                value
            );

            let value: f64 = setting_temperature.fahrenheit;
            SETTING_TEMPERATURE
                .with_label_values(&[zone.name.as_str(), device_type.as_str(), "fahrenheit"])
                .set(value);
            info!(
                "-> {} ({}) -> setting temperature (fahrenheit): {}",
                zone.name,
                device_type.as_str(),
                value
            );
        } else {
            for unit in ["celsius", "fahrenheit"] {
                // An error only means that the series does not exist yet.
                let _ = SETTING_TEMPERATURE.remove_label_values(&[
                    zone.name.as_str(),
                    device_type.as_str(),
                    unit,
                ]);
            }
            info!(
                "-> {} ({}) -> setting temperature (celsius): Off",
                zone.name,
                device_type.as_str()
            );
            info!(
                "-> {} ({}) -> setting temperature (fahrenheit): Off",
                zone.name,
                device_type.as_str()
            );
        }

        // If openWindowDetected is not None, this means that a window is open.
        if zone.state_response.openWindow.is_some() {
            info!(
                "-> {} ({}) -> window opened: {}",
                zone.name,
                device_type.as_str(),
                true
            );
            SENSOR_WINDOW_OPENED
                .with_label_values(&[zone.name.as_str(), device_type.as_str()])
                .set(1.0);
        } else {
            info!(
                "-> {} ({}) -> window opened: {}",
                zone.name,
                device_type.as_str(),
                false
            );
            SENSOR_WINDOW_OPENED
                .with_label_values(&[zone.name.as_str(), device_type.as_str()])
                .set(0.0);
        }

        // sensor temperature
        if let Some(inside_temperature) = zone.state_response.sensorDataPoints.insideTemperature {
            // celsius
            let value: f64 = inside_temperature.celsius;
            SENSOR_TEMPERATURE
                .with_label_values(&[zone.name.as_str(), device_type.as_str(), "celsius"])
                .set(value);
            info!(
                "-> {} ({}) -> sensor temperature (celsius): {}",
                zone.name,
                device_type.as_str(),
                value
            );

            // fahrenheit
            let value: f64 = inside_temperature.fahrenheit;
            SENSOR_TEMPERATURE
                .with_label_values(&[zone.name.as_str(), device_type.as_str(), "fahrenheit"])
                .set(value);
            info!(
                "-> {} ({}) -> sensor temperature (fahrenheit): {}",
                zone.name,
                device_type.as_str(),
                value
            );
        }

        // sensor humidity
        if let Some(humidity) = zone.state_response.sensorDataPoints.humidity {
            let value: f64 = humidity.percentage;
            SENSOR_HUMIDITY_PERCENTAGE
                .with_label_values(&[zone.name.as_str(), device_type.as_str()])
                .set(value);
            info!(
                "-> {} ({}) -> sensor humidity: {}%",
                zone.name,
                device_type.as_str(),
                value
            );
        }

        // heating power
        if let Some(heating_power) = zone.state_response.activityDataPoints.heatingPower {
            let value: f64 = heating_power.percentage;
            ACTIVITY_HEATING_POWER
                .with_label_values(&[zone.name.as_str(), device_type.as_str()])
                .set(value);
            info!(
                "-> {} ({}) -> heating power: {}%",
                zone.name,
                device_type.as_str(),
                value
            );
        }

        // ac power
        if let Some(ac_power) = zone.state_response.activityDataPoints.acPower {
            let value: f64 = match ac_power.value.as_str() {
                "ON" => 1.0,
                "OFF" => 0.0,
                _ => 0.0,
            };

            ACTIVITY_AC_POWER
                .with_label_values(&[zone.name.as_str(), device_type.as_str()])
                .set(value);
            info!(
                "-> {} ({}) -> ac power: {}",
                zone.name,
                device_type.as_str(),
                value
            );
        }
    }
}

pub fn set_weather(weather_response: Option<WeatherApiResponse>) {
    if let Some(weather) = weather_response {
        // setting solar intensity
        let solar_intensity_percentage = weather.solarIntensity.percentage;

        WEATHER_SOLAR_INTENSITY
            .with_label_values::<&str>(&[])
            .set(weather.solarIntensity.percentage);
        info!(
            "-> setting solar intensity (percentage): {}",
            solar_intensity_percentage
        );

        // setting outside temperature
        let outside_temperature_celsius = weather.outsideTemperature.celsius;
        let outside_temperature_fahrenheit = weather.outsideTemperature.fahrenheit;

        WEATHER_OUTSIDE_TEMPERATURE
            .with_label_values(&["celsius"])
            .set(outside_temperature_celsius);
        info!(
            "-> setting outside temperature (celsius): {}",
            outside_temperature_celsius
        );

        WEATHER_OUTSIDE_TEMPERATURE
            .with_label_values(&["fahrenheit"])
            .set(outside_temperature_fahrenheit);
        info!(
            "-> setting outside temperature (fahrenheit): {}",
            outside_temperature_fahrenheit
        );
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

#[cfg(test)]
mod tests {
    use crate::tado::model::{
        WeatherOutsideTemperatureApiResponse, WeatherSolarIntensityApiResponse,
        ZoneStateActivityDataPointsApiResponse, ZoneStateApiResponse,
        ZoneStateSensorDataPointsApiResponse, ZoneStateSettingApiResponse,
        ZoneStateSettingTemperatureApiResponse,
    };

    use super::*;

    fn weather_metrics() -> Vec<prometheus::proto::MetricFamily> {
        prometheus::gather()
            .into_iter()
            .filter(|family| family.name().starts_with("weather_"))
            .collect()
    }

    fn zone(name: &str, power: &str, celsius: Option<f64>) -> ZoneStateResponse {
        ZoneStateResponse {
            name: name.to_string(),
            state_response: ZoneStateApiResponse {
                setting: ZoneStateSettingApiResponse {
                    deviceType: "HOT_WATER".to_string(),
                    power: Some(power.to_string()),
                    temperature: celsius.map(|celsius| ZoneStateSettingTemperatureApiResponse {
                        celsius,
                        fahrenheit: celsius * 1.8 + 32.0,
                    }),
                },
                activityDataPoints: ZoneStateActivityDataPointsApiResponse {
                    heatingPower: None,
                    acPower: None,
                },
                sensorDataPoints: ZoneStateSensorDataPointsApiResponse {
                    insideTemperature: None,
                    humidity: None,
                },
                openWindow: None,
            },
        }
    }

    #[test]
    fn test_set_zones_power_off_removes_setting_temperature() {
        let setting_temperature_series = || {
            prometheus::gather()
                .iter()
                .filter(|family| family.name() == "tado_setting_temperature_value")
                .flat_map(|family| family.get_metric().iter())
                .filter(|metric| metric.get_label().iter().any(|l| l.value() == "Boiler"))
                .count()
        };

        // A zone that was never on has no series to remove.
        set_zones(vec![zone("Boiler", "OFF", None)]);
        assert_eq!(setting_temperature_series(), 0);

        set_zones(vec![zone("Boiler", "ON", Some(50.0))]);
        assert_eq!(
            SETTING_POWER
                .with_label_values(&["Boiler", "HOT_WATER"])
                .get(),
            1.0
        );
        assert_eq!(setting_temperature_series(), 2);

        set_zones(vec![zone("Boiler", "OFF", None)]);
        assert_eq!(
            SETTING_POWER
                .with_label_values(&["Boiler", "HOT_WATER"])
                .get(),
            0.0
        );
        assert_eq!(setting_temperature_series(), 0);
    }

    // The Prometheus registry is global: both scenarios live in a single test so
    // that they cannot race when the test harness runs on several threads.
    #[test]
    fn test_set_weather() {
        /*
        GIVEN no weather response
        WHEN set_weather is called
        THEN the metrics are not set
        */
        set_weather(None);
        assert_eq!(weather_metrics().len(), 0);

        /*
        GIVEN a weather response
        WHEN set_weather is called
        THEN the metrics are set
        */

        // GIVEN
        let weather_response = WeatherApiResponse {
            solarIntensity: WeatherSolarIntensityApiResponse { percentage: 100.0 },
            outsideTemperature: WeatherOutsideTemperatureApiResponse {
                celsius: 20.0,
                fahrenheit: 68.0,
            },
        };

        // WHEN
        set_weather(Some(weather_response));

        // THEN
        // Check metrics
        let metrics = weather_metrics();

        assert_eq!(metrics.len(), 2);
        assert_eq!(metrics[0].name(), "weather_outside_temperature");
        assert_eq!(metrics[1].name(), "weather_solar_intensity");

        // Check outside temperature metric
        let outside_temperature_metric = metrics[0].get_metric();

        assert_eq!(outside_temperature_metric.len(), 2);

        let outside_temp_celsius = &outside_temperature_metric[0];
        let outside_temp_fahrenheit = &outside_temperature_metric[1];

        assert_eq!(outside_temp_celsius.get_label().len(), 1);
        assert_eq!(outside_temp_celsius.get_label()[0].name(), "unit");
        assert_eq!(outside_temp_celsius.get_label()[0].value(), "celsius");
        assert_eq!(outside_temp_celsius.get_gauge().value(), 20.0);
        assert_eq!(outside_temp_fahrenheit.get_label().len(), 1);
        assert_eq!(outside_temp_fahrenheit.get_label()[0].name(), "unit");
        assert_eq!(outside_temp_fahrenheit.get_label()[0].value(), "fahrenheit");
        assert_eq!(outside_temp_fahrenheit.get_gauge().value(), 68.0);

        // Check solar intensity metric
        let solar_intensity_metric = metrics[1].get_metric();

        assert_eq!(solar_intensity_metric.len(), 1);
        assert_eq!(solar_intensity_metric[0].get_gauge().value(), 100.0);
    }
}
