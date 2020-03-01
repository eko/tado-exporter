use log::{info, error};
use std::vec::Vec;
use reqwest;

use super::model::{AuthApiResponse, MeApiResponse, ZonesApiResponse, ZoneStateApiResponse, ZoneStateResponse};

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

    pub async fn retrieve(&mut self) -> Vec<ZoneStateResponse> {
        // retrieve an access token to use the tado API
        let api_response = match self.authenticate().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("unable to authenticate: {}", e);
                return Vec::new();
            }
        };

        self.access_token = api_response.access_token;

        // retrieve home details (only if we don't already have a home identifier)
        if self.home_id == 0 {
            let me_response = match self.me().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("unable to retrieve home identifier: {}", e);
                    return Vec::new();
                }
            };

            self.home_id = me_response.homes.first().unwrap().id;
        }

        // retrieve home different zones
        let zones_response = match self.zones().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("unable to retrieve home zones: {}", e);
                return Vec::new();
            }
        };

        let mut response = Vec::<ZoneStateResponse>::new();

        for zone in zones_response {
            info!("retrieving zone details for {}...", zone.name);
            let zone_state_response = match self.zone_state(zone.id).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("unable to retrieve home zone '{}' state: {}", zone.name, e);
                    return Vec::new();
                }
            };

            response.push(ZoneStateResponse{
                name: zone.name,
                state_response: zone_state_response,
            });
        }

        return response;
    }
}
