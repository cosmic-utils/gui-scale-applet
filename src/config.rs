use std::fmt::Display;

use cosmic::cosmic_config::{Config, ConfigGet, ConfigSet};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub const APP_ID: &str = "com.bhh32.GUIScaleApplet";
pub const CONFIG_VERS: u64 = 2;

/// All user-configurable preferences, persisted across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    /// Selected exit node index.
    pub exit_node_idx: Option<usize>,
    /// Allow LAN access on exit node.
    pub allow_lan: bool,
    /// SSH is enabled.
    pub ssh_enabled: bool,
    /// Routes accepted.
    pub routes_accepted: bool,
    /// Auto-connect on applet startup.
    pub auto_connect: bool,
    /// Custom download directory for TailDrop.
    pub download_dir: Option<String>,
    /// Status pooling interval in seconds.
    pub poll_interval_secs: u64,
    /// Notifications enabled/disabled.
    pub notifications_enabled: bool,
    /// Notifiy on connection changed.
    pub notify_on_connection_change: bool,
    /// Notify on incoming files.
    pub notify_on_incoming_files: bool,
    /// Notify when a new device joins.
    pub notify_on_new_device: bool,
    /// Panel icon style: "dynamic" (changes with status) or "static".
    pub icon_style: String,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            exit_node_idx: None,
            allow_lan: false,
            ssh_enabled: false,
            routes_accepted: false,
            auto_connect: false,
            download_dir: None,
            poll_interval_secs: 10,
            notifications_enabled: true,
            notify_on_connection_change: true,
            notify_on_incoming_files: true,
            notify_on_new_device: true,
            icon_style: "dynamic".to_string(),
        }
    }
}

pub fn update_config<T>(config: Config, key: &str, value: T)
where
    T: Serialize + Display + Clone,
{
    let config_set = config.set(key, value.clone());

    match config_set {
        Ok(_) => println!("Config variable for {key} was set to {value}"),
        Err(e) => eprintln!("Something went wrong setting {key} to {value}: {e}"),
    }

    let config_tx = config.transaction();
    let tx_result = config_tx.commit();

    match tx_result {
        Ok(_) => println!("Config transaction has been completed!"),
        Err(e) => eprintln!("Something with the config transaction when wrong: {e}"),
    }
}

pub fn load_config<T>(key: &str, config_vers: u64) -> (Option<T>, String)
where
    T: DeserializeOwned,
{
    let config = match Config::new(APP_ID, config_vers) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Loading config file had an error: {e}");
            match Config::system(APP_ID, 1) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("System config also failed: {e}");
                    return (None, format!("Config error: {e}"));
                }
            }
        }
    };

    match config.get(key) {
        Ok(value) => (Some(value), "".to_owned()),
        Err(_) => {
            update_config(config, key, "");
            (None, "Created config for key".to_owned())
        }
    }
}

/// Load the full preferences struct from config.
pub fn load_preferences() -> AppPreferences {
    let mut prefs = AppPreferences::default();

    if let (Some(val), _) = load_config::<Option<usize>>("exit-node", CONFIG_VERS) {
        prefs.exit_node_idx = val;
    }
    if let (Some(val), _) = load_config::<bool>("allow-lan", CONFIG_VERS) {
        prefs.allow_lan = val;
    }
    if let (Some(val), _) = load_config::<bool>("ssh-enabled", CONFIG_VERS) {
        prefs.ssh_enabled = val;
    }
    if let (Some(val), _) = load_config::<bool>("routes-accepted", CONFIG_VERS) {
        prefs.routes_accepted = val;
    }
    if let (Some(val), _) = load_config::<bool>("auto-connect", CONFIG_VERS) {
        prefs.auto_connect = val;
    }
    if let (Some(val), _) = load_config::<String>("download-dir", CONFIG_VERS) {
        prefs.download_dir = Some(val);
    }
    if let (Some(val), _) = load_config::<u64>("poll-interval", CONFIG_VERS) {
        prefs.poll_interval_secs = val;
    }
    if let (Some(val), _) = load_config::<bool>("notifications-enabled", CONFIG_VERS) {
        prefs.notifications_enabled = val;
    }
    if let (Some(val), _) = load_config::<bool>("notify-connection", CONFIG_VERS) {
        prefs.notify_on_connection_change = val;
    }
    if let (Some(val), _) = load_config::<bool>("notify-files", CONFIG_VERS) {
        prefs.notify_on_incoming_files = val;
    }
    if let (Some(val), _) = load_config::<bool>("notify-device", CONFIG_VERS) {
        prefs.notify_on_new_device = val;
    }
    if let (Some(val), _) = load_config::<String>("icon-style", CONFIG_VERS) {
        prefs.icon_style = val;
    }

    prefs
}
