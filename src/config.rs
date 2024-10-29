use std::fmt::Display;

use cosmic::cosmic_config::{Config, ConfigGet, ConfigSet};
use serde::{de::DeserializeOwned, Serialize};

pub fn update_config<T>(config: Config, key: &str, value: T) where T: Serialize + Display + Clone {
    let config_set = config.set(key, value.clone());

    match config_set {
        Ok(_) => println!("Config variable for {key} was set to {value}"),
        Err(e) => println!("Something went wrong setting {key} to {value}"),
    }

    let config_tx = config.transaction();
    let tx_result = config_tx.commit();

    match tx_result {
        Ok(_) => println!("Config transaction has been completed!"),
        Err(e) => eprintln!("Something with the config transaction when wrong: {e}"),
    }
}

pub fn load_exit_node<T>(key: &str) -> T where T: DeserializeOwned {
    let config = match Config::new("com.github.bhh32.GUIScaleApplet", 1) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Loading config file had an error: {e}");
            Config::system("com.github.bhh32.GUIScaleApplet", 1).unwrap()
        }
    };

    match config.get(key) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Could not return value for {key}: {e}");
            panic!();
        }
    }
}