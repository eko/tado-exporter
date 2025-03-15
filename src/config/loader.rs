use std::env;

pub struct Config {
    pub ticker: u64,
    pub username: String,
    pub password: String,
    pub client_id: String,
}

impl Config {
    pub fn print(&self) {
        println!("--- tado° exporter configuration ---");
        println!("Ticker seconds: {}", self.ticker);
        println!("Username: {}", self.username);
        println!("Password: <not printed>");
        println!("Client ID: {}", self.client_id);
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
        client_id: match env::var("EXPORTER_CLIENT_ID") {
            Ok(v) => v,
            Err(_) => {
                "1bb50063-6b0c-4d11-bd99-387f4a91cc46".to_string()
            }
        },
    };

    config.print();

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        // Given no env variable are set
        env::remove_var("EXPORTER_USERNAME");
        env::remove_var("EXPORTER_PASSWORD");
        env::remove_var("EXPORTER_TICKER");
        env::remove_var("EXPORTER_CLIENT_ID");

        // when
        let config = load();

        // then we should load default values
        assert_eq!(config.ticker, 10);
        assert_eq!(config.username, "");
        assert_eq!(config.password, "");
        assert_eq!(
            config.client_id,
            "1bb50063-6b0c-4d11-bd99-387f4a91cc46"
        );

        // given the following environment variable values
        env::set_var("EXPORTER_USERNAME", "test-user");
        env::set_var("EXPORTER_PASSWORD", "123Password!");
        env::set_var("EXPORTER_TICKER", "30");
        env::set_var("EXPORTER_CLIENT_ID", "client-123");

        // when
        let config = load();

        // then we should have these values set
        assert_eq!(config.ticker, 30);
        assert_eq!(config.username, "test-user");
        assert_eq!(config.password, "123Password!");
        assert_eq!(config.client_id, "client-123");
    }
}
