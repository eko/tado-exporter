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
        println!("Password: <not printed>");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        // when
        let config = load();

        // then we should load default values
        assert_eq!(config.ticker, 10);
        assert_eq!(config.username, "");
        assert_eq!(config.password, "");
        assert_eq!(config.client_secret, "wZaRN7rpjn3FoNyF5IFuxg9uMzYJcvOoQ8QWiIqS3hfk6gLhVlG57j5YNoZL2Rtc");

        // given the following environment variable values
        env::set_var("EXPORTER_USERNAME", "test-user");
        env::set_var("EXPORTER_PASSWORD", "123Password!");
        env::set_var("EXPORTER_TICKER", "30");
        env::set_var("EXPORTER_CLIENT_SECRET", "123-secret");

        // when
        let config = load();

        // then we should have these values set
        assert_eq!(config.ticker, 30);
        assert_eq!(config.username, "test-user");
        assert_eq!(config.password, "123Password!");
        assert_eq!(config.client_secret, "123-secret");
    }
}
