use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, ConfigGet, ConfigSet, CosmicConfigEntry};

const NAME: &str = "com.github.bhh32.GUIScaleApplet";
const VERSION: u64 = 1;
const EXIT_NODE: &str = "exit-node";

#[must_use]
#[derive(Debug, Clone)]
pub struct Config {
    pub cosmic_config: Option<cosmic_config::Config>,
    pub exit_node: Box<String>,
}

impl ConfigSet for Config {
    fn set<T: serde::Serialize>(&self, key: &str, value: T) -> Result<(), cosmic_config::Error> {
        match key {
            "exit-node" => serde::Serialize::cd
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exit_node: String::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();

        let cfg = match cosmic_config::Config::new(NAME, VERSION) {
            Ok(cfg) => cfg,
            Err(why) => {
                tracing::warn!(?why, "failed to get config");
                return Self::default();
            }
        };

        if let Ok(exit_node) = cfg.get::<String>(EXIT_NODE) {
            config.exit_node = exit_node;
        }

        config
    }

    pub fn set_active_exit_node(&mut self, exit_node: String) {
        self.exit_node = exit_node;
    }
}
