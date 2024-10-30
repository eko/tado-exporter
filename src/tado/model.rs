use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AuthApiResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct MeApiResponse {
    pub homes: Vec<HomesApiResponse>,
}

#[derive(Deserialize, Debug)]
pub struct HomesApiResponse {
    pub id: i32,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateApiResponse {
    pub id: i32,
    pub name: String,
    pub setting: ZoneStateSettingApiResponse,
    pub sensorDataPoints: ZoneStateSensorDataPointsApiResponse,
    pub heatingPower: Option<ActivityDataPointsHeatingPowerApiResponse>,
    // pub openWindow: Option<ZoneStateOpenWindowApiResponse>,
}

// #[derive(Deserialize, Debug, PartialEq, Eq)]
// #[allow(non_snake_case)]
// pub struct ZoneStateOpenWindowApiResponse {
//     pub detectedTime: String, // RFC 3339 timestamp
//     pub durationInSeconds: i32,
//     pub expiry: String,
//     pub remainingTimeInSeconds: i32,
// }

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateSettingApiResponse {
    pub power: String,
    pub temperature: Option<ZoneStateSettingTemperatureApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ZoneStateSettingTemperatureApiResponse {
    pub value: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ActivityDataPointsHeatingPowerApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateSensorDataPointsApiResponse {
    pub insideTemperature: Option<SensorDataPointsInsideTemperatureApiResponse>,
    pub humidity: Option<SensorDataPointsHumidityApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SensorDataPointsInsideTemperatureApiResponse {
    pub value: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SensorDataPointsHumidityApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct WeatherApiResponse {
    pub solarIntensity: WeatherSolarIntensityApiResponse,
    pub outsideTemperature: WeatherOutsideTemperatureApiResponse,
}
#[derive(Deserialize, Debug, PartialEq)]
pub struct WeatherSolarIntensityApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct WeatherOutsideTemperatureApiResponse {
    pub fahrenheit: f64,
    pub celsius: f64,
}
