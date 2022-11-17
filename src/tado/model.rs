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

#[derive(Deserialize, Debug)]
pub struct ZonesApiResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateApiResponse {
    pub setting: ZoneStateSettingApiResponse,
    pub activityDataPoints: ZoneStateActivityDataPointsApiResponse,
    pub sensorDataPoints: ZoneStateSensorDataPointsApiResponse,
    // pub openWindow: Option<ZoneStateOpenWindowApiResponse>,
    pub openWindowDetection: Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateSettingApiResponse {
    #[serde(rename = "type")]
    pub deviceType: String,
    pub temperature: Option<ZoneStateSettingTemperatureApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ZoneStateSettingTemperatureApiResponse {
    pub celsius: f64,
    pub fahrenheit: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateActivityDataPointsApiResponse {
    pub heatingPower: Option<ActivityDataPointsHeatingPowerApiResponse>,
    pub acPower: Option<ActivityDataPointsAcPowerApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ActivityDataPointsHeatingPowerApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ActivityDataPointsAcPowerApiResponse {
    pub value: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateSensorDataPointsApiResponse {
    pub insideTemperature: Option<SensorDataPointsInsideTemperatureApiResponse>,
    pub humidity: Option<SensorDataPointsHumidityApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SensorDataPointsInsideTemperatureApiResponse {
    pub celsius: f64,
    pub fahrenheit: f64,
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

pub struct ZoneStateResponse {
    pub name: String,
    pub state_response: ZoneStateApiResponse,
}
