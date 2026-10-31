use std::time::{Duration, Instant};
use std::vec::Vec;

use lazy_static::lazy_static;
use log::{error, info};
use reqwest;

use super::error::AuthError;
use super::model::{
    AuthStartResponse, AuthTokensErrorResponse, AuthTokensResponse, MeApiResponse,
    WeatherApiResponse, ZoneStateApiResponse, ZoneStateResponse, ZonesApiResponse,
};

const AUTH_PENDING_MESSAGE: &str = "authorization_pending";

lazy_static! {
    // TODO: POST DEVICE - https://login.tado.com/oauth2/device
    static ref AUTH_START_URL: reqwest::Url = "https://login.tado.com/oauth2/device_authorize".parse().unwrap();
    static ref AUTH_TOKEN_URL: reqwest::Url = "https://login.tado.com/oauth2/token".parse().unwrap();
    pub static ref BASE_URL: reqwest::Url = "https://my.tado.com/api/v2/".parse().unwrap();
}

pub struct Client {
    http_client: reqwest::Client,
    base_url: reqwest::Url,

    // API Authentication information.
    client_id: String,
    tokens: AuthTokensResponse,
    tokens_refresh_by: Instant,

    home_id: i32,
}

impl Client {
    pub fn new(client_id: String) -> Client {
        Client::with_base_url(BASE_URL.clone(), client_id)
    }

    fn with_base_url(base_url: reqwest::Url, client_id: String) -> Client {
        Client {
            http_client: reqwest::Client::new(),
            base_url,
            client_id,
            tokens: AuthTokensResponse {
                access_token: String::default(),
                expires_in: 0,
                refresh_token: String::default(),
            },
            tokens_refresh_by: Instant::now(),
            home_id: 0,
        }
    }

    /// Authenticate to the Tado API service.
    ///
    /// The authentication processes uses the oauth2 device code grant flow as required by Tado
    /// <https://support.tado.com/en/articles/8565472-how-do-i-authenticate-to-access-the-rest-api>.
    /// The user has to approve the login by visiting the URL written to the logs.
    pub async fn authenticate(&mut self) -> Result<(), AuthError> {
        // Start device authentication flow.
        let start_params = [
            ("client_id", self.client_id.as_str()),
            ("scope", "offline_access"),
        ];
        let resp = self
            .http_client
            .post(AUTH_START_URL.clone())
            .form(&start_params)
            .send()
            .await?;
        let start = resp.json::<AuthStartResponse>().await?;
        info!(
            "Started device authentication flow with URL {}",
            start.verification_uri_complete
        );

        // Wait for API tokens to be returned once the flow is complete.
        self.wait_for_tokens(start).await?;
        Ok(())
    }

    async fn get(&self, url: reqwest::Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.tokens.access_token),
            )
            .send()
            .await
    }

    async fn me(&self) -> Result<MeApiResponse, reqwest::Error> {
        let url = self.base_url.join("/api/v2/me").unwrap();
        let resp = self.get(url).await?;

        resp.json::<MeApiResponse>().await
    }

    async fn zones(&mut self) -> Result<Vec<ZonesApiResponse>, reqwest::Error> {
        let endpoint = format!("/api/v2/homes/{}/zones", self.home_id);
        let url = self.base_url.join(&endpoint).unwrap();

        let resp = self.get(url).await?;

        resp.json::<Vec<ZonesApiResponse>>().await
    }

    async fn zone_state(&mut self, zone_id: i32) -> Result<ZoneStateApiResponse, reqwest::Error> {
        let endpoint = format!("/api/v2/homes/{}/zones/{}/state", self.home_id, zone_id);
        let url = self.base_url.join(&endpoint).unwrap();

        let resp = self.get(url).await?;

        resp.json::<ZoneStateApiResponse>().await
    }

    async fn weather(&self) -> Result<WeatherApiResponse, reqwest::Error> {
        let endpoint = format!("homes/{}/weather/", self.home_id);
        let url = self.base_url.join(&endpoint).unwrap();

        let resp = self.get(url).await?;

        resp.json::<WeatherApiResponse>().await
    }

    /// Refresh the API access token if it expired.
    pub async fn refresh_authentication(&mut self) -> Result<(), AuthError> {
        if Instant::now() < self.tokens_refresh_by {
            return Ok(());
        }

        let refresh_params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", self.tokens.refresh_token.as_str()),
        ];
        let resp = self
            .http_client
            .post(AUTH_TOKEN_URL.clone())
            .form(&refresh_params)
            .send()
            .await?;

        let tokens = resp.json::<AuthTokensResponse>().await?;
        self.set_tokens(tokens);
        info!("API access tokens refreshed");
        Ok(())
    }

    pub async fn retrieve_zones(&mut self) -> Vec<ZoneStateResponse> {
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

            response.push(ZoneStateResponse {
                name: zone.name,
                state_response: zone_state_response,
            });
        }

        response
    }

    pub async fn retrieve_weather(&mut self) -> Option<WeatherApiResponse> {
        info!("retrieving weather details ...");

        // retrieve home details (only if we don't already have a home identifier)
        if self.home_id == 0 {
            let me_response = match self.me().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("unable to retrieve home identifier: {}", e);
                    return None;
                }
            };

            self.home_id = me_response.homes.first().unwrap().id;
        }

        // retrieve weather state
        let weather_response = match self.weather().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("unable to retrieve weather info: {}", e);
                return None;
            }
        };

        Some(weather_response)
    }

    /// Set the API access tokens to use and manage related metadata.
    fn set_tokens(&mut self, tokens: AuthTokensResponse) {
        // Reduce the tokens validity slightly to refresh before they expire.
        let expires_in = tokens.expires_in - 10;
        self.tokens = tokens;
        self.tokens_refresh_by = Instant::now() + Duration::from_secs(expires_in);
    }

    async fn wait_for_tokens(&mut self, start: AuthStartResponse) -> Result<(), AuthError> {
        let must_complete_by = Instant::now() + Duration::from_secs(start.expires_in);
        let token_params = [
            ("client_id", self.client_id.as_str()),
            ("device_code", &start.device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];
        while Instant::now() < must_complete_by {
            let resp = self
                .http_client
                .post(AUTH_TOKEN_URL.clone())
                .form(&token_params)
                .send()
                .await?;
            match resp.status() {
                reqwest::StatusCode::OK => {
                    let tokens = resp.json::<AuthTokensResponse>().await?;
                    self.set_tokens(tokens);
                    info!("Device authentication flow completed");
                    return Ok(());
                }
                reqwest::StatusCode::BAD_REQUEST => {
                    let error = resp
                        .error_for_status_ref()
                        .expect_err("must be error for BAD_REQUEST");
                    let failure = resp.json::<AuthTokensErrorResponse>().await?;
                    if failure.error != AUTH_PENDING_MESSAGE {
                        return Err(AuthError::from(error));
                    }
                }
                _ => {
                    let status = resp.status();
                    let url = resp.url().clone();
                    resp.error_for_status()?;
                    return Err(AuthError::UnexpectedStatus(status, url));
                }
            }
            info!("Device authentication flow still pending, will retry");
            tokio::time::sleep(Duration::from_secs(start.interval)).await;
        }
        Err(AuthError::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::tado::model::{
        ActivityDataPointsHeatingPowerApiResponse, SensorDataPointsHumidityApiResponse,
        SensorDataPointsInsideTemperatureApiResponse, WeatherOutsideTemperatureApiResponse,
        WeatherSolarIntensityApiResponse, ZoneStateActivityDataPointsApiResponse,
        ZoneStateApiResponse, ZoneStateOpenWindowApiResponse, ZoneStateSensorDataPointsApiResponse,
        ZoneStateSettingApiResponse, ZoneStateSettingTemperatureApiResponse,
    };

    use rstest::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_new() {
        let client = Client::new("client_id".to_string());

        assert_eq!(client.client_id, "client_id");
        assert_eq!(client.base_url, *BASE_URL);
    }

    #[test]
    fn test_with_base_url() {
        let client = Client::with_base_url(
            "https://example.com".parse().unwrap(),
            "client_id".to_string(),
        );

        assert_eq!(client.client_id, "client_id");
        assert_eq!(client.base_url, "https://example.com".parse().unwrap());
    }

    #[rstest(response_str, expected,
        case(
            r#"
            {
                "solarIntensity": {
                  "type": "PERCENTAGE",
                  "percentage": 18.3,
                  "timestamp": "2022-09-03T17:43:41.088Z"
                },
                "outsideTemperature": {
                  "celsius": 21.53,
                  "fahrenheit": 70.75,
                  "timestamp": "2022-09-03T17:43:41.088Z",
                  "type": "TEMPERATURE",
                  "precision": { "celsius": 0.01, "fahrenheit": 0.01 }
                },
                "weatherState": {
                  "type": "WEATHER_STATE",
                  "value": "CLOUDY_PARTLY",
                  "timestamp": "2022-09-03T17:43:41.088Z"
                }
              }
            "#,
            WeatherApiResponse {
                solarIntensity: WeatherSolarIntensityApiResponse {
                    percentage: 18.3,
                },
                outsideTemperature: WeatherOutsideTemperatureApiResponse{
                    celsius: 21.53,
                    fahrenheit: 70.75
                },
            }
        )
    )]
    #[actix_rt::test]
    async fn test_weather(response_str: &str, expected: WeatherApiResponse) {
        /*
        GIVEN an OSM client
        WHEN calling the capabilities() function
        THEN returns the sets of capablities and policies
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("homes/0/weather/"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/json"))
            .mount(&mock_server)
            .await;

        let client =
            Client::with_base_url(mock_server.uri().parse().unwrap(), "client_id".to_string());

        // WHEN
        let actual = client.weather().await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }

    #[rstest(response_str, expected,
        case(
            r#"{
                "setting":{
                  "type":"tado",
                  "temperature":{
                    "celsius":21.53,
                    "fahrenheit":70.75
                  }
                },
                "activityDataPoints":{
                  "heatingPower":{
                    "percentage":0.0
                  },
                  "acPower":null
                },
                "sensorDataPoints":{
                  "insideTemperature":{
                    "celsius":25.0,
                    "fahrenheit":77.0
                  },
                  "humidity":{
                    "percentage":75.0
                  }
                }
              }"#,
            ZoneStateApiResponse {
                setting : ZoneStateSettingApiResponse {
                    deviceType: "tado".to_string(),
                    power: None,
                    temperature: Some(ZoneStateSettingTemperatureApiResponse {
                        celsius: 21.53,
                        fahrenheit: 70.75
                    })
                },
                activityDataPoints : ZoneStateActivityDataPointsApiResponse {
                    heatingPower : Some(ActivityDataPointsHeatingPowerApiResponse {
                        percentage: 0.0
                    }),
                    acPower : None
                },
                openWindow: None,
                sensorDataPoints: ZoneStateSensorDataPointsApiResponse {
                    insideTemperature : Some(SensorDataPointsInsideTemperatureApiResponse {
                        celsius: 25.0,
                        fahrenheit: 77.0
                    }),
                    humidity : Some(SensorDataPointsHumidityApiResponse {
                        percentage: 75.0
                    })
                }
            }
        ),
        case(
            r#"{
                "setting":{
                  "type":"tado",
                  "temperature":{
                    "celsius":21.53,
                    "fahrenheit":70.75
                  }
                },
                "openWindow":{
                    "detectedTime":"2022-11-21T11:15:32Z",
                    "durationInSeconds":900,
                    "expiry":"2022-11-21T11:30:32Z",
                    "remainingTimeInSeconds":662
                },
                "activityDataPoints":{
                  "heatingPower":{
                    "percentage":0.0
                  },
                  "acPower":null
                },
                "sensorDataPoints":{
                  "insideTemperature":{
                    "celsius":25.0,
                    "fahrenheit":77.0
                  },
                  "humidity":{
                    "percentage":75.0
                  }
                }
              }"#,
            ZoneStateApiResponse {
                setting : ZoneStateSettingApiResponse {
                    deviceType: "tado".to_string(),
                    power: None,
                    temperature: Some(ZoneStateSettingTemperatureApiResponse {
                        celsius: 21.53,
                        fahrenheit: 70.75
                    })
                },
                openWindow : Some(ZoneStateOpenWindowApiResponse {
                    detectedTime: "2022-11-21T11:15:32Z".to_string(),
                    durationInSeconds: 900,
                    expiry: "2022-11-21T11:30:32Z".to_string(),
                    remainingTimeInSeconds: 662
                }),
                activityDataPoints : ZoneStateActivityDataPointsApiResponse {
                    heatingPower : Some(ActivityDataPointsHeatingPowerApiResponse {
                        percentage: 0.0
                    }),
                    acPower : None
                },
                sensorDataPoints: ZoneStateSensorDataPointsApiResponse {
                    insideTemperature : Some(SensorDataPointsInsideTemperatureApiResponse {
                        celsius: 25.0,
                        fahrenheit: 77.0
                    }),
                    humidity : Some(SensorDataPointsHumidityApiResponse {
                        percentage: 75.0
                    })
                }
            }
        ),
        // A hot water zone has no sensor and no activity data, and it has no
        // temperature when it is turned off.
        case(
            r#"{
                "setting":{
                  "type":"HOT_WATER",
                  "power":"OFF",
                  "temperature":null
                },
                "activityDataPoints":{},
                "sensorDataPoints":{}
              }"#,
            ZoneStateApiResponse {
                setting : ZoneStateSettingApiResponse {
                    deviceType: "HOT_WATER".to_string(),
                    power: Some("OFF".to_string()),
                    temperature: None
                },
                activityDataPoints : ZoneStateActivityDataPointsApiResponse {
                    heatingPower : None,
                    acPower : None
                },
                openWindow: None,
                sensorDataPoints: ZoneStateSensorDataPointsApiResponse {
                    insideTemperature : None,
                    humidity : None
                }
            }
        )
    )]
    #[actix_rt::test]
    async fn test_zone_state(response_str: &str, expected: ZoneStateApiResponse) {
        /*
        GIVEN an OSM client
        WHEN calling the zone_state() function
        THEN returns the zone state
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("api/v2/homes/0/zones/0/state"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/json"))
            .mount(&mock_server)
            .await;

        let mut client =
            Client::with_base_url(mock_server.uri().parse().unwrap(), "client_id".to_string());

        // WHEN
        let actual = client.zone_state(0).await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
