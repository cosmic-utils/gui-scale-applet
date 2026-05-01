use http_body_util::{BodyExt, Full};
use hyper::{
    Request,
    body::Bytes,
    client::conn::http1::{SendRequest, handshake},
};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    io::ErrorKind,
    path::Path,
    time::Duration,
};
use tokio::net::UnixStream;

/// Default path to the tailscaled Unix socket.
const DEFAULT_SOCKET_PATH: &str = "/var/run/tailscale/tailscaled.sock";
/// The host header value expected by tailscaled.
const LOCAL_API_HOST: &str = "local-tailscaled.sock";

#[derive(Debug, Clone)]
pub enum TailscaleError {
    /// The tailscaled socket was not found; daemon likely not running.
    SocketNotFound,
    /// Connection to the socket was refused.
    ConnectionRefused(String),
    /// The HTTP request failed.
    RequestFailed(String),
    /// The response could not parsed.
    ParseError(String),
    /// The API returned an error status code.
    ApiError(u16, String),
    /// Operator permission not set.
    OperatorNotSet,
}

impl Display for TailscaleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TailscaleError::SocketNotFound => write!(
                f,
                "Tailscale daemon not found. Is tailscaled running?\n\
                 Socket not found at {DEFAULT_SOCKET_PATH}\n\
                 Start it with: sudo systemctl start tailscaled"
            ),
            TailscaleError::ConnectionRefused(err) => {
                write!(f, "Cannot connect to tailscaled: {err}")
            }
            TailscaleError::RequestFailed(err) => write!(f, "Request failed: {err}"),
            TailscaleError::ParseError(err) => write!(f, "Parse error: {err}"),
            TailscaleError::ApiError(code, body) => write!(f, "API error (HTTP {code}: {body}"),
            TailscaleError::OperatorNotSet => write!(
                f,
                "Tailscale operator not set for your user.\n\
                Run: sudo tailscale set --operator=$USER"
            ),
        }
    }
}

pub type TsResult<T> = Result<T, TailscaleError>;

/// Full status response from `/localapi/v0/status`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Status {
    /// Version of the tailscale backend.
    #[serde(default)]
    pub version: String,
    /// Whether or not the backend is running.
    #[serde(default)]
    pub backend_state: String,
    /// This node's info.
    #[serde(rename = "Self")]
    pub self_node: Option<PeerStatus>,
    /// Map of peer node key -> peer status.
    #[serde(default)]
    pub peer: HashMap<String, PeerStatus>,
    /// The current tailname name.
    #[serde(default)]
    pub current_tailnet: Option<TailnetStatus>,
    /// MagicDNS suffix for the tailnet.
    #[serde(default)]
    pub magic_dns_suffix: String,
}

/// Status of peer or self node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PeerStatus {
    /// Stable node ID.
    #[serde(rename = "ID", default)]
    pub id: String,
    /// Public key of the node.
    #[serde(default)]
    pub public_key: String,
    /// Hostname
    #[serde(default)]
    pub host_name: String,
    /// DNS name (FQDN).
    #[serde(rename = "DNSName", default)]
    pub dns_name: String,
    /// OS of the peer
    #[serde(rename = "OS", default)]
    pub os: String,
    /// Tailscale IP addresses
    #[serde(rename = "TailscaleIPs", default)]
    pub tailscale_ips: Vec<String>,
    /// Is the peer online.
    #[serde(default)]
    pub online: bool,
    /// Is the node an exit node.
    #[serde(default)]
    pub exit_node: bool,
    /// It is the currently used exit node.
    #[serde(default)]
    pub exit_node_option: bool,
    /// Tags assigned to this node.
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    /// DERP relay region.
    #[serde(default)]
    pub relay: String,
    /// Bytes received from this peer.
    #[serde(default)]
    pub rx_bytes: u64,
    /// Bytes sent to this peer.
    #[serde(default)]
    pub tx_bytes: u64,
    /// When the peer was last seen.
    #[serde(default)]
    pub last_seen: String,
    /// Does peer have the exit node capability.
    #[serde(default)]
    pub exit_node_option_enabled: bool,
    /// Is a Mullvad exit node.
    #[serde(default)]
    pub is_mullvad: bool,
}

/// Telnet Info
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct TailnetStatus {
    /// Name of the tailnet.
    #[serde(default)]
    pub name: String,
    /// MagicDNS suffix.
    #[serde(rename = "MagicDNSSuffix", default)]
    pub magic_dns_suffix: String,
    /// Is MagicDNS enabled?
    #[serde(rename = "MagicDNSEnabled", default)]
    pub magic_dns_enabled: bool,
}

/// Preferences from `/localapi/v0/prefs`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Prefs {
    /// Does the user want Tailscale Running.
    #[serde(default)]
    pub want_running: bool,
    /// SSH enabled?
    #[serde(rename = "RunSSH", default)]
    pub run_ssh: bool,
    /// Accept routes from other nodes.
    #[serde(default)]
    pub route_all: bool,
    /// Accept DNS from the tailnet.
    #[serde(rename = "CorpDNS", default)]
    pub corp_dns: bool,
    /// Exit node IP being used.
    #[serde(rename = "ExitNodeIP", default)]
    pub exit_node_ip: String,
    /// Allow LAN access when using as an exit node.
    #[serde(rename = "ExitNodeAllowLANAccess", default)]
    pub exit_not_allow_lan_access: bool,
    /// Routes being advertised.
    #[serde(default)]
    pub advertise_routes: Option<Vec<String>>,
    /// Hostname of this node.
    #[serde(default)]
    pub hostname: String,
    /// Operator user.
    #[serde(default)]
    pub operator_user: String,
}

/// Partial prefs update for PATCH /localapi/v0/prefs.
///
/// Tailscale's `MaskedPrefs` requires a `<Field>Set: true` flag for every
/// field being changed; the daemon silently ignores fields whose mask is
/// unset. The `*_set` booleans here are those mask flags. Use
/// [`PrefsUpdate::set_*`] helpers (or set both fields manually) — setting a
/// value without its mask is a no-op on the daemon.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PrefsUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub want_running: Option<bool>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub want_running_set: bool,

    #[serde(rename = "RunSSH", skip_serializing_if = "Option::is_none")]
    pub run_ssh: Option<bool>,
    #[serde(rename = "RunSSHSet", default, skip_serializing_if = "is_false")]
    pub run_ssh_set: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_all: Option<bool>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub route_all_set: bool,

    #[serde(rename = "CorpDNS", skip_serializing_if = "Option::is_none")]
    pub corp_dns: Option<bool>,
    #[serde(rename = "CorpDNSSet", default, skip_serializing_if = "is_false")]
    pub corp_dns_set: bool,

    #[serde(rename = "ExitNodeIP", skip_serializing_if = "Option::is_none")]
    pub exit_node_ip: Option<String>,
    #[serde(rename = "ExitNodeIPSet", default, skip_serializing_if = "is_false")]
    pub exit_node_ip_set: bool,

    #[serde(rename = "ExitNodeAllowLANAccess", skip_serializing_if = "Option::is_none")]
    pub exit_node_allow_lan_access: Option<bool>,
    #[serde(rename = "ExitNodeAllowLANAccessSet", default, skip_serializing_if = "is_false")]
    pub exit_node_allow_lan_access_set: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub advertise_routes: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub advertise_routes_set: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub hostname_set: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Profile (account) Info
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Profile {
    /// Profile ID.
    #[serde(rename = "ID", default)]
    pub id: String,
    /// Display name of the profile/account.
    #[serde(default)]
    pub name: String,
    /// The tailnet name.
    #[serde(default)]
    pub network_profile: Option<NetworkProfile>,
    /// The current profile
    #[serde(default)]
    pub current_profile: bool,
}

/// Network profile associated with a Profile.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkProfile {
    /// MagicDNS name of the tailnet.
    #[serde(rename = "MagicDNSName", default)]
    pub magic_dns_name: String,
    /// Domain name.
    #[serde(default)]
    pub domain_name: String,
}

/// A file waiting in the TailDrop inbox.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct WaitingFile {
    pub name: String,
    pub size: u64,
}

/// Ping result from `/localapi/v0/ping`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PingResult {
    /// The type of ping (TSMP, disco, ICMP).
    #[serde(rename = "Type", default)]
    pub ping_type: String,
    /// The IP that was pinged.
    #[serde(rename = "IP", default)]
    pub ip: String,
    /// Node IP pinged.
    #[serde(default)]
    pub node_ip: String,
    /// Name of the node.
    #[serde(default)]
    pub node_name: String,
    /// Latency in seconds.
    #[serde(default)]
    pub latency_seconds: f64,
    /// The endpoint used.
    #[serde(default)]
    pub endpoint: String,
    /// Is the connection direct (not relayed).
    #[serde(default)]
    pub is_direct: bool,
    /// Error message, if any.
    #[serde(default)]
    pub err: String,
}

/// One frame from the IPN notify bus. We only care about the auth URL field.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct Notify {
    #[serde(rename = "BrowseToURL", default)]
    browse_to_url: Option<String>,
}

/// A client for the Tailscale LocalAPI over Unix socket.
#[derive(Clone)]
pub struct TailscaleClient {
    socket_path: String,
}

impl TailscaleClient {
    /// Create a new client using the default socket path.
    pub fn new() -> Self {
        Self {
            socket_path: DEFAULT_SOCKET_PATH.to_string(),
        }
    }

    /// Send a GET request to the LocalAPI.
    async fn get(&self, path: &str) -> TsResult<String> {
        self.request("GET", path, None).await
    }

    /// Send a POSST request to the LocalAPI.
    async fn post(&self, path: &str, body: Option<String>) -> TsResult<String> {
        self.request("POST", path, body).await
    }

    /// Send a PATCH request to the LocalAPI.
    async fn patch(&self, path: &str, body: Option<String>) -> TsResult<String> {
        self.request("PATCH", path, body).await
    }

    /// Open a Unix-socket HTTP/1 connection to the LocalAPI and spawn the
    /// connection task. Returns the request `sender` ready to issue calls.
    async fn open_connection(&self) -> TsResult<SendRequest<Full<Bytes>>> {
        if !Path::new(&self.socket_path).exists() {
            return Err(TailscaleError::SocketNotFound);
        }

        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|err| {
                if err.kind() == ErrorKind::PermissionDenied {
                    TailscaleError::OperatorNotSet
                } else {
                    TailscaleError::ConnectionRefused(err.to_string())
                }
            })?;

        let io = TokioIo::new(stream);
        let (sender, conn) = handshake(io)
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("LocalAPI connection error: {e}");
            }
        });

        Ok(sender)
    }

    /// Core HTTP request over Unix socket.
    async fn request(&self, method: &str, path: &str, body: Option<String>) -> TsResult<String> {
        let mut sender = self.open_connection().await?;

        let uri = format!("http://{LOCAL_API_HOST}{path}");
        let req_body = match body {
            Some(body) => Full::new(Bytes::from(body)),
            None => Full::new(Bytes::new()),
        };

        let req = Request::builder()
            .method(method)
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .body(req_body)
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        // Send the request
        let response = sender
            .send_request(req)
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let status = response.status().as_u16();

        // Read the response body
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?
            .to_bytes();

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        if status == 403 {
            return Err(TailscaleError::ApiError(status, body_str));
        }

        Ok(body_str)
    }

    /// Get the full tailscale status.
    pub async fn status(&self) -> TsResult<Status> {
        let body = self.get("/localapi/v0/status").await?;
        serde_json::from_str(&body)
            .map_err(|err| TailscaleError::ParseError(format!("status: {err}")))
    }

    /// Get current preferences.
    pub async fn prefs(&self) -> TsResult<Prefs> {
        let body = self.get("/localapi/v0/prefs").await?;
        serde_json::from_str(&body)
            .map_err(|err| TailscaleError::ParseError(format!("prefs: {err}")))
    }

    /// Get available profiles (accounts).
    pub async fn profiles(&self) -> TsResult<Vec<Profile>> {
        let body = self.get("/localapi/v0/profiles/").await?;
        serde_json::from_str(&body)
            .map_err(|err| TailscaleError::ParseError(format!("profiles: {err}")))
    }

    /// Get the current profile.
    pub async fn current_profile(&self) -> TsResult<Profile> {
        let body = self.get("/localapi/v0/profiles/current").await?;
        serde_json::from_str(&body)
            .map_err(|err| TailscaleError::ParseError(format!("current profile: {err}")))
    }

    /// Update preferences
    /// Accepts a partial Prefs JSON and merges it.
    pub async fn set_prefs(&self, prefs: &PrefsUpdate) -> TsResult<Prefs> {
        let body = serde_json::to_string(prefs)
            .map_err(|err| TailscaleError::ParseError(err.to_string()))?;
        let response = self.patch("/localapi/v0/prefs", Some(body)).await?;
        serde_json::from_str(&response)
            .map_err(|err| TailscaleError::ParseError(format!("set_prefs response: {err}")))
    }

    /// Interactive login (opens browser).
    pub async fn login_interactive(&self) -> TsResult<()> {
        self.post("/localapi/v0/login-interactive", None).await?;
        Ok(())
    }

    /// Subscribe to the IPN notify bus and invoke `on_frame` for each parsed
    /// `Notify` frame. Returns when the stream is closed or errors; callers
    /// are responsible for reconnect/backoff. The bus opens with mask=14
    /// (`NotifyInitialState | NotifyInitialPrefs | NotifyInitialNetMap`) so
    /// the daemon emits current state immediately on connect.
    pub async fn run_ipn_bus_listener<F>(&self, mut on_frame: F) -> TsResult<()>
    where
        F: FnMut() + Send,
    {
        let mut sender = self.open_connection().await?;

        let uri = format!("http://{LOCAL_API_HOST}/localapi/v0/watch-ipn-bus?mask=14");
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .body(Full::new(Bytes::new()))
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let response = sender
            .send_request(req)
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let mut body = response.into_body();
        let mut buf: Vec<u8> = Vec::new();
        while let Some(frame) = body.frame().await {
            let frame = frame.map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;
            if let Ok(data) = frame.into_data() {
                buf.extend_from_slice(&data);
                while let Some(idx) = buf.iter().position(|b| *b == b'\n') {
                    let line: Vec<u8> = buf.drain(..=idx).collect();
                    let line = &line[..line.len() - 1];
                    if line.is_empty() {
                        continue;
                    }
                    if serde_json::from_slice::<Notify>(line).is_ok() {
                        on_frame();
                    }
                }
            }
        }
        Err(TailscaleError::RequestFailed(
            "IPN bus stream closed".to_string(),
        ))
    }

    /// Stream the IPN notify bus and return the first auth URL the daemon
    /// publishes (a `BrowseToURL` notification). Used to drive the new-account
    /// login flow inside a Flatpak sandbox where shelling out to the
    /// `tailscale` CLI isn't available.
    pub async fn wait_for_browse_url(&self, timeout: Duration) -> TsResult<String> {
        tokio::time::timeout(timeout, self.read_browse_url())
            .await
            .map_err(|_| {
                TailscaleError::RequestFailed("timed out waiting for auth URL".to_string())
            })?
    }

    async fn read_browse_url(&self) -> TsResult<String> {
        let mut sender = self.open_connection().await?;

        let uri = format!("http://{LOCAL_API_HOST}/localapi/v0/watch-ipn-bus?mask=0");
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .body(Full::new(Bytes::new()))
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let response = sender
            .send_request(req)
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let mut body = response.into_body();
        let mut buf: Vec<u8> = Vec::new();
        while let Some(frame) = body.frame().await {
            let frame = frame.map_err(|e| TailscaleError::RequestFailed(e.to_string()))?;
            if let Ok(data) = frame.into_data() {
                buf.extend_from_slice(&data);
                while let Some(idx) = buf.iter().position(|b| *b == b'\n') {
                    let line: Vec<u8> = buf.drain(..=idx).collect();
                    let line = &line[..line.len() - 1];
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(notify) = serde_json::from_slice::<Notify>(line) {
                        if let Some(url) = notify.browse_to_url {
                            if !url.is_empty() {
                                return Ok(url);
                            }
                        }
                    }
                }
            }
        }
        Err(TailscaleError::RequestFailed(
            "IPN bus closed before BrowseToURL was received".to_string(),
        ))
    }

    /// Switch to a different profile/account.
    pub async fn switch_profile(&self, profile_id: &str) -> TsResult<()> {
        self.post(&format!("/localapi/v0/profiles/{profile_id}"), None)
            .await?;

        Ok(())
    }

    /// Ping a peer.
    pub async fn ping(&self, ip: &str, ping_type: &str) -> TsResult<PingResult> {
        let body = self
            .post(&format!("/localapi/v0/ping?ip={ip}&type={ping_type}"), None)
            .await?;

        serde_json::from_str(&body)
            .map_err(|err| TailscaleError::ParseError(format!("ping: {err}")))
    }

    /// Send a file via TailDrop (PUT file content).
    pub async fn file_put(&self, peer_id: &str, filename: &str, content: Vec<u8>) -> TsResult<()> {
        let path = format!("/localapi/v0/file-put/{peer_id}/{filename}");

        let mut sender = self.open_connection().await?;

        let uri = format!("http://{LOCAL_API_HOST}{path}");
        let req = Request::builder()
            .method("PUT")
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .header("Content-Length", content.len())
            .body(Full::new(Bytes::from(content)))
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let response = sender
            .send_request(req)
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let status = response.status().as_u16();
        if status >= 400 {
            let body_bytes = response
                .into_body()
                .collect()
                .await
                .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?
                .to_bytes();

            let body_str = String::from_utf8_lossy(&body_bytes).to_string();
            return Err(TailscaleError::ApiError(status, body_str));
        }

        Ok(())
    }

    /// Retrieve waiting files from TailDrop inbox.
    pub async fn waiting_files(&self) -> TsResult<Vec<WaitingFile>> {
        let body = self.get("/localapi/v0/files/").await?;
        serde_json::from_str(&body).map_err(|err| {
            // Empty response means no files
            if body.is_empty() || body == "null" {
                return TailscaleError::ParseError("no files".into());
            }
            TailscaleError::ParseError(format!("files: {err}"))
        })
    }

    /// Download a specific file from TailDrop inbox.
    pub async fn file_get(&self, filename: &str) -> TsResult<Vec<u8>> {
        let mut sender = self.open_connection().await?;

        let uri = format!("http://{LOCAL_API_HOST}/localapi/v0/files/{filename}");
        let req = Request::builder()
            .method("GET")
            .uri(&uri)
            .header("Host", LOCAL_API_HOST)
            .body(Full::new(Bytes::new()))
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let response = sender
            .send_request(req)
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?;

        let status = response.status().as_u16();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|err| TailscaleError::RequestFailed(err.to_string()))?
            .to_bytes();

        if status >= 400 {
            return Err(TailscaleError::ApiError(
                status,
                String::from_utf8_lossy(&body_bytes).to_string(),
            ));
        }

        Ok(body_bytes.to_vec())
    }

    /// Delete a file from the TailDrop inbox (after downloading).
    pub async fn file_delete(&self, filename: &str) -> TsResult<()> {
        self.request("DELETE", &format!("/localapi/v0/files/{filename}"), None)
            .await?;

        Ok(())
    }

    /// Set the exit node to use. Pass empty string to clear.
    pub async fn set_exit_node(&self, node_ip: &str) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            exit_node_ip: Some(node_ip.to_string()),
            exit_node_ip_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Enable/disable this host as an exit node.
    pub async fn set_advertise_exit_node(&self, advertise: bool) -> TsResult<Prefs> {
        let routes = if advertise {
            vec!["0.0.0.0/0".to_string(), "::/0".to_string()]
        } else {
            vec![]
        };
        let prefs = PrefsUpdate {
            advertise_routes: Some(routes),
            advertise_routes_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set SSH enabled/disabled.
    pub async fn set_ssh(&self, enabled: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            run_ssh: Some(enabled),
            run_ssh_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set accept-routes enabled/disabled.
    pub async fn set_accept_routes(&self, accept: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            route_all: Some(accept),
            route_all_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set accept-dns (MagicDNS) enabled/disabled.
    pub async fn set_accept_dns(&self, accept: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            corp_dns: Some(accept),
            corp_dns_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set exit-node-allow-lan-access.
    pub async fn set_exit_node_allow_lan(&self, allow: bool) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            exit_node_allow_lan_access: Some(allow),
            exit_node_allow_lan_access_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Set advertised routes (subnet router)
    pub async fn set_advertise_routes(&self, routes: Vec<String>) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            advertise_routes: Some(routes),
            advertise_routes_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Connect (set WantRunning = true).
    pub async fn connect(&self) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            want_running: Some(true),
            want_running_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }

    /// Disconnect (set WantRunning = false).
    pub async fn disconnect(&self) -> TsResult<Prefs> {
        let prefs = PrefsUpdate {
            want_running: Some(false),
            want_running_set: true,
            ..Default::default()
        };
        self.set_prefs(&prefs).await
    }
}

impl Default for TailscaleClient {
    fn default() -> Self {
        Self::new()
    }
}
