use cosmic::cosmic_config::{self, ConfigGet, ConfigSet};

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
            "exit-node" => serde::Serialize::
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cosmic_config: None,
            exit_node: Box::from(String::new()),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        let mut config = Self::default();

        let ctx = match cosmic_config::Config::new(NAME, VERSION) {
            Ok(ctx) => ctx,
            Err(why) => {
                tracing::warn!(?why, "failed to get config");
                return Self::default();
            }
        };

        if let Ok(exit_node) = ctx.get::<Box<String>>(EXIT_NODE) {
            config.exit_node = exit_node;
        }

        config.cosmic_config = Some(ctx);

        config
    }

    pub fn set_active_exit_node(&mut self, exit_node: Box<String>) {
        if let Some(ctx) = self.cosmic_config.as_ref() {
            if let Err(why) = ctx.set::<Box<String>>(EXIT_NODE, exit_node.clone()) {
                tracing::error!(?why, "failed to store active exit node");
            }
        }

        self.exit_node = exit_node;

        let conf = self.clone();

        ConfigSet::set(&conf, EXIT_NODE, self.exit_node);
    }
}