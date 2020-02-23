extern crate reqwest;

use super::metrics::{TEMPERATURE_GAUGE, HUMIDITY_PERCENTAGE};
use super::model::{AuthApiResponse, MeApiResponse, ZonesApiResponse, ZoneStateApiResponse};

const AUTH_URL: &'static str = "https://auth.tado.com/oauth/token";

macro_rules! format_base_url {
    () => { "https://my.tado.com{endpoint}" };
}

pub struct Client {
    http_client: reqwest::Client,
    username: String,
    password: String,
    client_secret: String,
    access_token: String,
    home_id: i32,
}

impl Client {
    pub fn new(username: String, password: String, client_secret: String) -> Client {
        Client {
            http_client: reqwest::Client::new(),
            username: username,
            password: password,
            client_secret: client_secret,
            access_token: String::default(),
            home_id: 0,
        }
    }

    async fn authenticate(&mut self) -> Result<AuthApiResponse, reqwest::Error> {
        let params = [
            ("client_id", "tado-web-app"),
            ("client_secret", self.client_secret.as_str()),
            ("grant_type", "password"),
            ("scope", "home.user"),
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
        ];

        let resp = self.http_client
            .post(reqwest::Url::parse(AUTH_URL).unwrap())
            .form(&params)
            .send().await?;

        Ok(resp.json::<AuthApiResponse>().await?)
    }

    async fn get(&self, url: String) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(reqwest::Url::parse(url.as_str()).unwrap())
            .header("Authorization", format!("Bearer: {}", self.access_token))
            .send().await
    }

    async fn me(&self) -> Result<MeApiResponse, reqwest::Error> {
        let url = format!(format_base_url!(), endpoint = "/api/v2/me");
        let resp = self.get(url).await?;

        Ok(resp.json::<MeApiResponse>().await?)
    }

    async fn zones(&mut self) -> Result<Vec<ZonesApiResponse>, reqwest::Error> {
        let endpoint = format!("/api/v2/homes/{}/zones", self.home_id);
        let url = format!(format_base_url!(), endpoint = endpoint);

        let resp = self.get(url).await?;

        Ok(resp.json::<Vec<ZonesApiResponse>>().await?)
    }

    async fn zone_state(&mut self, zone_id: i32) -> Result<ZoneStateApiResponse, reqwest::Error> {
        let endpoint = format!("/api/v2/homes/{}/zones/{}/state", self.home_id, zone_id);
        let url = format!(format_base_url!(), endpoint = endpoint);

        let resp = self.get(url).await?;

        Ok(resp.json::<ZoneStateApiResponse>().await?)
    }

    pub async fn retrieve(&mut self) {
        // retrieve an access token to use the tado API
        let api_response = self.authenticate().await.unwrap();
        self.access_token = api_response.access_token;

        // retrieve home details (only if we don't already have a home identifier)
        if self.home_id == 0 {
            let me_response = self.me().await.unwrap();
            self.home_id = me_response.homes.first().unwrap().id;
        }

        // retrieve home different zones
        let zones_response = self.zones().await.unwrap();
        for zone in zones_response {
            println!("[tado° client] retrieving zone details for {}...", zone.name);
            let zone_state_response = self.zone_state(zone.id).await.unwrap();

            // temperature: celsius
            let temperature_celsius: f64 = zone_state_response.sensorDataPoints.insideTemperature.celsius;
            TEMPERATURE_GAUGE.with_label_values(&[zone.name.as_str(), "celsius"]).set(temperature_celsius);
            println!("-> temperature (celsius): {}", temperature_celsius);

            // temperature: fahrenheit
            let temperature_fahrenheit: f64 = zone_state_response.sensorDataPoints.insideTemperature.fahrenheit;
            TEMPERATURE_GAUGE.with_label_values(&[zone.name.as_str(), "fahrenheit"]).set(temperature_celsius);
            println!("-> temperature (fahrenheit): {}", temperature_fahrenheit);

            // humidity percentage
            let humidity_percentage: f64 = zone_state_response.sensorDataPoints.humidity.percentage;
            HUMIDITY_PERCENTAGE.with_label_values(&[zone.name.as_str()]).set(humidity_percentage);
            println!("-> humidity: {}%", humidity_percentage);
        }
    }
}
