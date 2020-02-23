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

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ZoneStateApiResponse {
    pub sensorDataPoints: ZoneStateSensorDataPointsApiResponse,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ZoneStateSensorDataPointsApiResponse {
    pub insideTemperature: SensorDataPointsInsideTemperatureApiResponse,
    pub humidity: SensorDataPointsHumidityApiResponse,
}

#[derive(Deserialize, Debug)]
pub struct SensorDataPointsInsideTemperatureApiResponse {
    pub celsius: f64,
    pub fahrenheit: f64,
}

#[derive(Deserialize, Debug)]
pub struct SensorDataPointsHumidityApiResponse {
    pub percentage: f64,
}
