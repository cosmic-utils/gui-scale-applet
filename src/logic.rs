use crate::tailscale_api::{PeerStatus, TailscaleClient, TsResult};
use std::{
    env,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

// Re-export the error types so window.rs can use them.
pub use crate::tailscale_api::{PingResult, TailscaleError, WaitingFile};

#[derive(Debug, Clone, Default)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub dns_name: String,
    pub tailscale_ips: Vec<String>,
    pub os: String,
    pub online: bool,
    pub is_exit_node: bool,
    pub exit_node_option: bool,
    pub tags: Vec<String>,
    pub relay: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub last_seen: String,
    pub is_self: bool,
}

impl From<(&PeerStatus, bool)> for DeviceInfo {
    fn from((peer, is_self): (&PeerStatus, bool)) -> Self {
        let name = peer
            .dns_name
            .split('.')
            .next()
            .unwrap_or(&peer.host_name)
            .to_string();

        DeviceInfo {
            id: peer.id.clone(),
            name,
            dns_name: peer.dns_name.trim_end_matches('.').to_string(),
            tailscale_ips: peer.tailscale_ips.clone(),
            os: peer.os.clone(),
            online: if is_self { true } else { peer.online },
            is_exit_node: peer.exit_node,
            exit_node_option: peer.exit_node_option,
            tags: peer.tags.clone().unwrap_or_default(),
            relay: peer.relay.clone(),
            rx_bytes: peer.rx_bytes,
            tx_bytes: peer.tx_bytes,
            last_seen: peer.last_seen.clone(),
            is_self,
        }
    }
}

/// Account/profile info.
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
    pub tailnet: String,
    pub is_current: bool,
}

/// Full snapshot of Tailscale state.
#[derive(Debug, Clone, Default)]
pub struct TailscaleState {
    pub connected: bool,
    pub ssh_enabled: bool,
    pub accept_routes: bool,
    pub magic_dns: bool,
    pub is_exit_node: bool,
    pub exit_node_allow_lan: bool,
    pub ip_v4: String,
    pub ip_v6: String,
    pub dns_suffix: String,
    pub devices: Vec<DeviceInfo>,
    pub device_names: Vec<String>,
    pub exit_node_options: Vec<DeviceInfo>,
    pub accounts: Vec<AccountInfo>,
    pub advertised_routes: Vec<String>,
    pub waiting_files: Vec<WaitingFile>,
}

/// Fetch a complete snapshot of Tailscale state.
/// Used for both init and periodic polling.
pub async fn fetch_state(client: &TailscaleClient) -> TsResult<TailscaleState> {
    let status = client.status().await?;
    let prefs = client.prefs().await?;

    // Parse self node
    let self_node = status.self_node.as_ref();
    let ip_v4 = self_node
        .and_then(|node| node.tailscale_ips.first())
        .cloned()
        .unwrap_or_else(|| "N/A".to_string());
    let ip_v6 = self_node
        .and_then(|node| node.tailscale_ips.get(1))
        .cloned()
        .unwrap_or_else(|| "N/A".to_string());

    // Build device list. status.peer is a HashMap with non-deterministic
    // iteration order — sort peers by name so the dropdown index stays
    // stable across fetches (otherwise the selection appears to "cycle"
    // whenever a new IPN notify triggers a refetch).
    let mut devices = Vec::new();
    if let Some(ref self_peer) = status.self_node {
        devices.push(DeviceInfo::from((self_peer, true)));
    }

    let mut peers: Vec<DeviceInfo> = status
        .peer
        .values()
        .map(|peer| DeviceInfo::from((peer, false)))
        .collect();
    peers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    devices.extend(peers);

    // Device names to TailDrop dropdown
    let mut device_names = vec!["Select".to_string()];
    for dev in &devices {
        if !dev.is_self {
            device_names.push(dev.name.clone());
        }
    }

    // Exit node operations
    let exit_node_options: Vec<DeviceInfo> = devices
        .iter()
        .filter(|dev| dev.exit_node_option && !dev.is_self)
        .cloned()
        .collect();

    // This this host an exit node?
    let is_exit_node = prefs
        .advertise_routes
        .as_ref()
        .map(|routes| routes.iter().any(|rt| rt == "0.0.0.0/0" || rt == "::/0"))
        .unwrap_or(false);

    // Advertised routes (excluding exit node routes)
    let advertised_routes: Vec<String> = prefs
        .advertise_routes
        .as_ref()
        .map(|routes| {
            routes
                .iter()
                .filter(|route| *route != "0.0.0.0/0" && *route != "::/0")
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    // Accounts
    let profiles = client.profiles().await.unwrap_or_default();
    let current_profile = client.current_profile().await.ok();

    let accounts: Vec<AccountInfo> = profiles
        .iter()
        .map(|profile| {
            let is_current = current_profile
                .as_ref()
                .map(|cur_prof| cur_prof.id == profile.id)
                .unwrap_or(false);

            AccountInfo {
                id: profile.id.clone(),
                name: profile.name.clone(),
                tailnet: profile
                    .network_profile
                    .as_ref()
                    .map(|net_prof| net_prof.domain_name.clone())
                    .unwrap_or_default(),
                is_current,
            }
        })
        .collect();

    let dns_suffix = status
        .current_tailnet
        .as_ref()
        .map(|tailnet| tailnet.magic_dns_suffix.clone())
        .unwrap_or_default();

    let waiting_files = client.waiting_files().await.unwrap_or_default();

    Ok(TailscaleState {
        connected: prefs.want_running,
        ssh_enabled: prefs.run_ssh,
        accept_routes: prefs.route_all,
        magic_dns: prefs.corp_dns,
        is_exit_node,
        exit_node_allow_lan: prefs.exit_not_allow_lan_access,
        ip_v4,
        ip_v6,
        dns_suffix,
        devices,
        device_names,
        exit_node_options,
        accounts,
        advertised_routes,
        waiting_files,
    })
}

/// Connect to the tailnet.
pub async fn connect(client: &TailscaleClient) -> TsResult<()> {
    client.connect().await?;
    Ok(())
}

/// Disconnect from the tailnet.
pub async fn disconnect(client: &TailscaleClient) -> TsResult<()> {
    client.disconnect().await?;
    Ok(())
}

/// Set connection state.
pub async fn set_connected(client: &TailscaleClient, connected: bool) -> TsResult<()> {
    if connected {
        connect(client).await
    } else {
        disconnect(client).await
    }
}

/// Set SSH enabled/disabled.
pub async fn set_ssh(client: &TailscaleClient, enabled: bool) -> TsResult<()> {
    client.set_ssh(enabled).await?;
    Ok(())
}

/// Set accept-routes enabled/disabled.
pub async fn set_routes(client: &TailscaleClient, accept: bool) -> TsResult<()> {
    client.set_accept_routes(accept).await?;
    Ok(())
}

/// Set MagicDNS enabled/disabled.
pub async fn set_magic_dns(client: &TailscaleClient, enabled: bool) -> TsResult<()> {
    client.set_accept_dns(enabled).await?;
    Ok(())
}

/// Set exit node by IP. Pass empty string to clear.
pub async fn set_exit_node(client: &TailscaleClient, node_ip: &str) -> TsResult<()> {
    client.set_exit_node(node_ip).await?;
    Ok(())
}

/// Enable/disable this host as an exit node.
pub async fn set_advertise_exit_node(client: TailscaleClient, advertise: bool) -> TsResult<()> {
    client.set_advertise_exit_node(advertise).await?;
    Ok(())
}

/// Seet exit-node-allow-lan-access.
pub async fn set_exit_node_allow_lan(client: &TailscaleClient, allow: bool) -> TsResult<()> {
    client.set_exit_node_allow_lan(allow).await?;
    Ok(())
}

/// Switch to a different account/profile.
pub async fn switch_account(client: &TailscaleClient, profile_id: &str) -> TsResult<()> {
    client.switch_profile(profile_id).await
}

/// Open login page in browser for a new account.
///
/// Drives the IPN-bus flow end-to-end: subscribe to the notify bus, ask the
/// daemon to start an interactive login, then `xdg-open` the URL the daemon
/// publishes. Inside a Flatpak sandbox, `xdg-open` is routed through the
/// OpenURI portal so the host browser receives the URL.
pub async fn login_new_account(client: &TailscaleClient) -> TsResult<()> {
    let watcher = client.wait_for_browse_url(std::time::Duration::from_secs(30));
    client.login_interactive().await?;
    let url = watcher.await?;
    open_url(&url)
}

fn open_url(url: &str) -> TsResult<()> {
    use std::process::{Command, Stdio};
    Command::new("xdg-open")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|e| TailscaleError::RequestFailed(format!("xdg-open: {e}")))
}

/// Ping a device by IP.
pub async fn ping_device(client: &TailscaleClient, ip: &str) -> TsResult<PingResult> {
    client.ping(ip, "disco").await
}

/// Set advertised subnet routes.
pub async fn set_advertised_routes(client: &TailscaleClient, routes: Vec<String>) -> TsResult<()> {
    client.set_advertise_routes(routes).await?;
    Ok(())
}

/// Send a file to a peer via TailDrop (LocalAPI file-put).
pub async fn send_file(
    client: &TailscaleClient,
    peer_id: &str,
    file_path: &str,
) -> Result<(), String> {
    let path = Path::new(file_path);
    let filename = path
        .file_name()
        .and_then(|node| node.to_str())
        .ok_or_else(|| format!("Invalid filename: {file_path}"))?;
    let content = tokio::fs::read(path)
        .await
        .map_err(|err| format!("Failed to read {file_path}: {err}"))?;

    client
        .file_put(peer_id, filename, content)
        .await
        .map_err(|err| format!("Failed to send {filename}: {err}"))
}

/// Send multiple files to a peer.
pub async fn send_files(
    client: &TailscaleClient,
    peer_id: &str,
    file_paths: &[String],
) -> Option<String> {
    let mut errors = Vec::new();

    for path in file_paths {
        if let Err(e) = send_file(client, peer_id, path).await {
            errors.push(e);
        }
    }

    if errors.is_empty() {
        None
    } else {
        Some(errors.join("\n"))
    }
}

/// Receive all waiting files from TailDrop inbox.
pub async fn receive_files(
    client: &TailscaleClient,
    download_dir: &str,
) -> Result<Vec<String>, String> {
    let waiting = client
        .waiting_files()
        .await
        .map_err(|err| format!("Failed to list waiting files: {err}"))?;

    if waiting.is_empty() {
        return Err("No files waiting in TailDrop inbox.".to_string());
    }

    // Ensure download dir exists
    tokio::fs::create_dir_all(download_dir)
        .await
        .map_err(|err| format!("Failed to create download dir: {err}"))?;

    let mut received = Vec::new();

    for file in &waiting {
        let content = client
            .file_get(&file.name)
            .await
            .map_err(|err| format!("Failed to download {}: {err}", file.name))?;

        let dest = format!("{}/{}", download_dir.trim_end_matches('/'), file.name);
        tokio::fs::write(&dest, &content)
            .await
            .map_err(|err| format!("Failed to write {dest}: {err}"))?;

        // Delete from inbox after successful download
        let _ = client.file_delete(&file.name).await;
        received.push(file.name.clone());
    }

    Ok(received)
}

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Failed to run wl-copy: {err}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|err| format!("Failed to write to clipboard: {err}"))?;
    }

    child
        .wait()
        .map_err(|err| format!("wl-copy failed: {err}"))?;

    Ok(())
}

// Non-blocking sleep for status clearing.
pub async fn clear_status(wait_time: u64) -> Option<String> {
    tokio::time::sleep(Duration::from_secs(wait_time)).await;
    None
}

/// Format bytes into human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Get the default download directory for the current user.
pub fn default_download_dir() -> String {
    if let Ok(home) = env::var("HOME") {
        format!("{home}/Downloads")
    } else {
        "/tmp".to_string()
    }
}
