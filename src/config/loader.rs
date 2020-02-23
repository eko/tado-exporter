use std::env;

pub struct Config {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn print(&self) {
        println!("--- tado° exporter configuration ---");
        println!("Username: {}", self.username);
        println!("Password: {}", self.password);
        println!("------------------------------------");
    }
}

pub fn load() -> Config {
    let config = Config {
        username: match env::var("EXPORTER_USERNAME") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        },
        password: match env::var("EXPORTER_PASSWORD") {
            Ok(v) => v,
            Err(_) => "".to_string(),
        },
    };

    config.print();

    return config;
}
