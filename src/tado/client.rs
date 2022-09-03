use lazy_static::lazy_static;
use log::{error, info};
use reqwest;
use std::vec::Vec;

use super::model::{AuthApiResponse, MeApiResponse, ZonesApiResponse, ZoneStateApiResponse, ZoneStateResponse};

const AUTH_URL: &'static str = "https://auth.tado.com/oauth/token";

lazy_static! {
    pub static ref BASE_URL: reqwest::Url = "https://my.tado.com/".parse().unwrap();
}

pub struct Client {
    http_client: reqwest::Client,
    base_url: reqwest::Url,
    username: String,
    password: String,
    client_secret: String,
    access_token: String,
    home_id: i32,
}

impl Client {
    pub fn new(username: String, password: String, client_secret: String) -> Client {
        Client::with_base_url(BASE_URL.clone(), username, password, client_secret)
    }

    fn with_base_url(
        base_url: reqwest::Url,
        username: String,
        password: String,
        client_secret: String,
    ) -> Client {
        Client {
            http_client: reqwest::Client::new(),
            base_url,
            username,
            password,
            client_secret,
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

    async fn get(&self, url: reqwest::Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header("Authorization", format!("Bearer: {}", self.access_token))
            .send().await
    }

    async fn me(&self) -> Result<MeApiResponse, reqwest::Error> {
        let url = self.base_url.join("/api/v2/me").unwrap();
        let resp = self.get(url).await?;

        Ok(resp.json::<MeApiResponse>().await?)
    }

    async fn zones(&mut self) -> Result<Vec<ZonesApiResponse>, reqwest::Error> {
        let endpoint = format!("/api/v2/homes/{}/zones", self.home_id);
        let url = self.base_url.join(&endpoint).unwrap();

        let resp = self.get(url).await?;

        Ok(resp.json::<Vec<ZonesApiResponse>>().await?)
    }

    async fn zone_state(&mut self, zone_id: i32) -> Result<ZoneStateApiResponse, reqwest::Error> {
        let endpoint = format!("/api/v2/homes/{}/zones/{}/state", self.home_id, zone_id);
        let url = self.base_url.join(&endpoint).unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let client = Client::new(
            "username".to_string(),
            "password".to_string(),
            "client_secret".to_string(),
        );

        assert_eq!(client.username, "username");
        assert_eq!(client.password, "password");
        assert_eq!(client.client_secret, "client_secret");
        assert_eq!(client.base_url, *BASE_URL);
    }

    #[test]
    fn test_with_base_url() {
        let client = Client::with_base_url(
            "https://example.com".parse().unwrap(),
            "username".to_string(),
            "password".to_string(),
            "client_secret".to_string(),
        );

        assert_eq!(client.username, "username");
        assert_eq!(client.password, "password");
        assert_eq!(client.client_secret, "client_secret");
        assert_eq!(client.base_url, "https://example.com".parse().unwrap());
    }
}
