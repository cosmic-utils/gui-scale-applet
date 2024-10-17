use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, ConfigGet, ConfigSet, CosmicConfigEntry};

const NAME: &str = "com.github.bhh32.GUIScaleApplet";
const VERSION: u64 = 1;
const EXIT_NODE: &str = "exit-node";

#[must_use]
#[derive(Debug, Clone, CosmicConfigEntry)]
pub struct Config<'a> {
    pub name: &'a str,
    pub vers: u64,
    pub exit_node: String,
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