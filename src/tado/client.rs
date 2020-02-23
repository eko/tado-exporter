extern crate reqwest;

use super::metrics::TEMPERATURE_GAUGE;

const BASE_URL: &str = "https://my.tado.com";

pub struct Client {
    pub httpClient: reqwest::Client,
    pub username: String,
    pub password: String,
}

impl Client {
    pub fn authenticate(&self) {
        // let mut res = reqwest::get("http://httpbin.org/get")?;
        // let mut body = String::new();
        // res.read_to_string(&mut body)?;

        // println!("Status: {}", res.status());
        // println!("Headers:\n{:#?}", res.headers());
        // println!("Body:\n{}", body);

        println!("OK auth");
        TEMPERATURE_GAUGE.set(22 as f64);

        // Ok(())
    }
}
