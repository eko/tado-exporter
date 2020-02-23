use std::env;

pub struct Config {
    pub ticker: u64,
    pub username: String,
    pub password: String,
    pub client_secret: String,
}

impl Config {
    pub fn print(&self) {
        println!("--- tado° exporter configuration ---");
        println!("Ticker seconds: {}", self.ticker);
        println!("Username: {}", self.username);
        println!("Password: {}", self.password);
        println!("Client secret: {}", self.client_secret);
        println!("------------------------------------");
    }
}

pub fn load() -> Config {
    let config = Config {
        ticker: match env::var("EXPORTER_TICKER") {
            Ok(v) => v.parse::<u64>().unwrap(),
            Err(_) => 10,
        },
        username: match env::var("EXPORTER_USERNAME") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        },
        password: match env::var("EXPORTER_PASSWORD") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        },
        client_secret: match env::var("EXPORTER_CLIENT_SECRET") {
            Ok(v) => v,
            Err(_) => "wZaRN7rpjn3FoNyF5IFuxg9uMzYJcvOoQ8QWiIqS3hfk6gLhVlG57j5YNoZL2Rtc".to_string(),
        },
    };

    config.print();

    return config;
}
