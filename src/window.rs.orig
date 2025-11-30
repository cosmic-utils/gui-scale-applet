<<<<<<< HEAD
use crate::{
    config::{APP_ID, AppPreferences, CONFIG_VERS, load_preferences, update_config},
    fl,
    logic::{
        PingResult, TailscaleState, clear_status, copy_to_clipboard, default_download_dir,
        fetch_state, format_bytes, login_new_account, ping_device, receive_files, send_files,
        set_advertise_exit_node, set_advertised_routes, set_connected, set_exit_node,
        set_exit_node_allow_lan, set_magic_dns, set_routes, set_ssh, switch_account,
    },
    notifications::*,
    tailscale_api::TailscaleClient,
=======
use crate::config::{load_config, update_config};
use crate::logic::{
    clear_status, enable_exit_node, exit_node_allow_lan_access, get_acct_list,
    get_avail_exit_nodes, get_current_acct, get_is_exit_node, get_tailscale_con_status,
    get_tailscale_devices, get_tailscale_ip, get_tailscale_routes_status, get_tailscale_ssh_status,
    set_exit_node, set_routes, set_ssh, switch_accounts, tailscale_int_up, tailscale_recieve,
    tailscale_send,
>>>>>>> a2689dd (Add account switching)
};
use cosmic::{
    Action, Element, Task,
    app::Core,
    cosmic_config::Config,
    dialog::file_chooser::{self, FileFilter},
    iced::{
        self, Alignment, Length, Limits, Subscription,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column, row},
        window::Id,
    },
    iced_runtime::core::window,
    task,
    widget::{
        button, container, dropdown, icon, list_column, scrollable, settings, text, text_input,
        toggler,
    },
};
use std::{fmt::Debug, time::Duration};
use url::Url;

const POPUP_MAX_WIDTH: f32 = 1440.0;
const POPUP_MIN_WIDTH: f32 = 480.0;
const POPUP_MAX_HEIGHT: f32 = 720.0;
const POPUP_MIN_HEIGHT: f32 = 640.0;
const STATUS_CLEAR_TIME: u64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Status,
    TailDrop,
    Devices,
    Settings,
}

/// Holds the applet's state
pub struct Window {
    core: Core,
    config: Config,
    client: TailscaleClient,
    popup: Option<Id>,
    state: TailscaleState,
    active_tab: Tab,
    selected_device_idx: Option<usize>,
    selected_device_name: String,
    send_files: Vec<String>,
    send_file_status: String,
    files_sent: bool,
    receive_file_status: String,
    exit_node_names: Vec<String>,
    sel_exit_node_idx: Option<usize>,
<<<<<<< HEAD
    acct_names: Vec<String>,
    selected_device_detail_idx: Option<usize>,
    ping_result: Option<PingResult>,
    ping_in_progress: bool,
    subnet_input: String,
    preferences: AppPreferences,
    previous_connected_state: bool,
    previous_device_count: usize,
    notifications_initialized: bool,
    initial_load_done: bool,
=======
    acct_list: Vec<String>,
    cur_acct: String,
    allow_lan: bool,
    is_exit_node: bool,
>>>>>>> a2689dd (Add account switching)
}

/// Messages to be sent to the Libcosmic Update function
#[derive(Clone, Debug)]
pub enum Message {
    // Popup
    TogglePopup,
    PopupClosed(Id),
    TabSelected(Tab),

    // Polling
    IpnEvent,
    StateLoaded(Result<TailscaleState, String>),

    // Connection
    EnableSSH(bool),
    AcceptRoutes(bool),
    ConnectDisconnect(bool),
<<<<<<< HEAD
    ToggleMagicDns(bool),

    // Accounts
    SwitchAccount(usize),
    LoginNewAccount,

    // Tails Drop
=======
    SwitchAccount(usize),
>>>>>>> a2689dd (Add account switching)
    DeviceSelected(usize),
    ChooseFiles,
    FilesSelected(Vec<Url>),
    SendFiles,
    FilesSent(Option<String>),
    FileChoosingCancelled,
    RecieveFiles,
    FilesRecieved(String),
    ClearTailDropStatus,

    // Exit Node
    ExitNodeSelected(usize),
    AllowExitNodeLanAccess(bool),
    UpdateIsExitNode(bool),

    // Device details
    SelectDeviceDetail(usize),
    PingDevice(String),
    PingCompleted(Result<PingResult, String>),
    CopyToClipboard(String),

    // Subnets
    SubnetInput(String),
    AddSubnet,
    RemoveSubnet(usize),

    // Settings
    SetAutoConnect(bool),
    SetNotificationsEnabled(bool),
    SetNotifyConnection(bool),
    SetNotifyFiles(bool),
    SetNotifyDevice(bool),
    SetIconStyle(bool),
    ChooseDownloadDir,
    DownloadDirSelected(Vec<Url>),
    DownloadDirCancelled,

    ActionCompleted(Result<(), String>),
}

impl cosmic::Application for Window {
    type Executor = cosmic::executor::multi::Executor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Window, Task<Action<Self::Message>>) {
<<<<<<< HEAD
        let client = TailscaleClient::new();
        let preferences = load_preferences();
=======
        // Get the SSH status from the Tailscale CLI
        let ssh = get_tailscale_ssh_status();
        // Get the Accept Routes status from the Tailscale CLI
        let routes = get_tailscale_routes_status();
        // Get the connection status from the Tailscale CLI
        let connect = get_tailscale_con_status();
        // Get the other devices on the Tailnet from the Tailscale CLI
        let device_options = get_tailscale_devices();

        // Set the default applet state for allow_lan to false
        let allow_lan = false;
        // Get the state of the host being an exit node from the Tailscale CLI
        let is_exit_node = get_is_exit_node();

        // Get the list of accounts the device is registered on
        let acct_list = get_acct_list();

        // Get which account the device is currently logged into
        let cur_acct = get_current_acct();

        // Check to see if the host is an exit node already.
        // If it's not, get the available exit nodes.
        // If it is, set exit_nodes_init to the messag.
        let exit_nodes_init = if !is_exit_node {
            get_avail_exit_nodes()
        } else {
            vec![String::from(
                "Can't select an exit node\nwhile host is an exit node!",
            )]
        };
>>>>>>> a2689dd (Add account switching)

        // Set the start up state of the application using the above variables
        let window = Window {
            core,
<<<<<<< HEAD
            config: Config::new(APP_ID, CONFIG_VERS).unwrap(),
            client: client.clone(),
=======
            config: Config::new(ID, CONFIG_VERS).unwrap(),
            ssh,
            routes,
            connect,
            device_options,
>>>>>>> a2689dd (Add account switching)
            popup: None,
            state: TailscaleState::default(),
            active_tab: Tab::Status,
            selected_device_idx: Some(0),
            selected_device_name: fl!("select-default"),
            send_files: Vec::new(),
            send_file_status: String::new(),
            files_sent: false,
<<<<<<< HEAD
            receive_file_status: String::new(),
            exit_node_names: vec![fl!("none-default")],
            sel_exit_node_idx: preferences.exit_node_idx,
            acct_names: Vec::new(),
            selected_device_detail_idx: None,
            ping_result: None,
            ping_in_progress: false,
            subnet_input: String::new(),
            preferences,
            previous_connected_state: false,
            previous_device_count: 0,
            notifications_initialized: false,
            initial_load_done: false,
=======
            recieve_file_status: String::new(),
            avail_exit_nodes: exit_nodes_init,
            sel_exit_node: DEFAULT_EXIT_NODE.to_string(),
            sel_exit_node_idx: None,
            acct_list,
            cur_acct,
            allow_lan,
            is_exit_node,
>>>>>>> a2689dd (Add account switching)
        };

        // Kick off the initial async state load
        let init_client = client;
        let task = cosmic::task::future(async move {
            match fetch_state(&init_client).await {
                Ok(state) => Message::StateLoaded(Ok(state)),
                Err(e) => Message::StateLoaded(Err(e.to_string())),
            }
        });

        (window, task)
    }

    // The function that is called when the applet is closed
    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let client = self.client.clone();
        Subscription::run_with_id(
            "ipn-bus",
            iced::stream::channel(64, move |output| {
                let client = client.clone();
                async move {
                    loop {
                        let mut sender = output.clone();
                        let result = client
                            .run_ipn_bus_listener(move || {
                                let _ = sender.try_send(Message::IpnEvent);
                            })
                            .await;
                        if let Err(e) = result {
                            eprintln!("IPN bus listener disconnected: {e}");
                        }
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }),
        )
    }

    // Libcosmic's update function
    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        let mut tasks: Vec<Task<Action<Message>>> = Vec::new();
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    self.receive_file_status = String::new();
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(POPUP_MAX_WIDTH)
                        .min_width(POPUP_MIN_WIDTH)
                        .min_height(POPUP_MIN_HEIGHT)
                        .max_height(POPUP_MAX_HEIGHT);

                    get_popup(popup_settings)
                };
            }
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::IpnEvent => {
                let client = self.client.clone();
                tasks.push(task::future(async move {
                    match fetch_state(&client).await {
                        Ok(state) => Message::StateLoaded(Ok(state)),
                        Err(e) => Message::StateLoaded(Err(e.to_string())),
                    }
                }));
            }
            Message::StateLoaded(result) => {
                match result {
                    Ok(new_state) => {
                        // Notifications for state changes
                        if self.notifications_initialized && self.preferences.notifications_enabled
                        {
                            if self.preferences.notify_on_connection_change
                                && new_state.connected != self.previous_connected_state
                            {
                                notify_connection_change(new_state.connected);
                            }

                            if self.preferences.notify_on_new_device
                                && new_state.devices.len() > self.previous_device_count
                            {
                                for dev in &new_state.devices {
                                    if !self.state.devices.iter().any(|device| device.id == dev.id)
                                    {
                                        notify_new_device(&dev.name);
                                    }
                                }
                            }

                            if self.preferences.notify_on_incoming_files
                                && !new_state.waiting_files.is_empty()
                            {
                                notify_incoming_files();
                            }
                        }

                        self.previous_connected_state = new_state.connected;
                        self.previous_device_count = new_state.devices.len();
                        self.notifications_initialized = true;

                        // Update derived UI state. Format account labels as
                        // "name (tailnet)" so users can disambiguate when the
                        // same name spans multiple tailnets.
                        self.acct_names = new_state
                            .accounts
                            .iter()
                            .map(|acct| {
                                if acct.tailnet.is_empty() {
                                    acct.name.clone()
                                } else {
                                    format!("{} ({})", acct.name, acct.tailnet)
                                }
                            })
                            .collect();

                        // Exit node dropdown names
                        let mut en_names = vec![fl!("none-default")];
                        for dev in &new_state.exit_node_options {
                            en_names.push(dev.name.clone());
                        }
                        self.exit_node_names = en_names;

                        // Auto-connect on first load if configured
                        if !self.initial_load_done
                            && self.preferences.auto_connect
                            && !new_state.connected
                        {
                            let client = self.client.clone();
                            tasks.push(task::future(async move {
                                let _ = set_connected(&client, true).await;
                                Message::ActionCompleted(Ok(()))
                            }));
                        }

                        self.initial_load_done = true;
                        self.state = new_state;
                    }
                    Err(e) => {
                        if e.contains("not found") || e.contains("Socket") {
                            eprintln!("tailscaled socket not found");
                        } else if e.contains("operator") || e.contains("403") {
                            eprintln!("tailscale operator not set");
                        } else {
                            eprintln!("Error: {e}");
                        }
                    }
                }
            }
            Message::EnableSSH(enabled) => {
                let client = self.client.clone();
                tasks.push(task::future(async move {
                    match set_ssh(&client, enabled).await {
                        Ok(()) => Message::ActionCompleted(Ok(())),
                        Err(e) => Message::ActionCompleted(Err(format!("set_ssh: {e}"))),
                    }
                }));
            }
            Message::AcceptRoutes(accepted) => {
                let client = self.client.clone();
                tasks.push(task::future(async move {
                    match set_routes(&client, accepted).await {
                        Ok(()) => Message::ActionCompleted(Ok(())),
                        Err(e) => Message::ActionCompleted(Err(format!("set_routes: {e}"))),
                    }
                }));
            }
            Message::ConnectDisconnect(connection) => {
                let client = self.client.clone();
                let notify = self.preferences.notifications_enabled
                    && self.preferences.notify_on_connection_change;
                tasks.push(task::future(async move {
                    let result = set_connected(&client, connection).await;
                    if notify && result.is_ok() {
                        notify_connection_change(connection);
                    }
                    match result {
                        Ok(()) => Message::ActionCompleted(Ok(())),
                        Err(e) => Message::ActionCompleted(Err(format!("set_connected: {e}"))),
                    }
                }));
            }
            Message::ToggleMagicDns(enabled) => {
                let client = self.client.clone();
                tasks.push(task::future(async move {
                    match set_magic_dns(&client, enabled).await {
                        Ok(()) => Message::ActionCompleted(Ok(())),
                        Err(e) => Message::ActionCompleted(Err(format!("set_magic_dns: {e}"))),
                    }
                }));
            }
            Message::SwitchAccount(new_acct) => {
                if let Some(acct) = self.state.accounts.get(new_acct) {
                    let client = self.client.clone();
                    let profile_id = acct.id.clone();
                    let acct_name = acct.name.clone();
                    let notify = self.preferences.notifications_enabled;
                    tasks.push(task::future(async move {
                        let _ = switch_account(&client, &profile_id).await;
                        if notify {
                            notify_account_switched(&acct_name);
                        }
                        Message::ActionCompleted(Ok(()))
                    }));
                }
            }
            Message::LoginNewAccount => {
                let client = self.client.clone();
                tasks.push(task::future(async move {
                    match login_new_account(&client).await {
                        Ok(()) => Message::ActionCompleted(Ok(())),
                        Err(e) => Message::ActionCompleted(Err(e.to_string())),
                    }
                }));
            }
            Message::SwitchAccount(new_acct) => {
                self.cur_acct = self.acct_list[new_acct].clone();
                switch_accounts(self.cur_acct.clone());

                self.ssh = get_tailscale_ssh_status();
                set_ssh(self.ssh);
                self.routes = get_tailscale_routes_status();
                set_routes(self.routes);
                self.device_options = get_tailscale_devices();
                self.avail_exit_nodes = get_avail_exit_nodes();
            }
            Message::DeviceSelected(device) => {
                self.selected_device_idx = Some(device);
                self.selected_device_name = self
                    .state
                    .device_names
                    .get(device)
                    .cloned()
                    .unwrap_or_else(|| fl!("select-default"));

                if self.files_sent {
                    self.files_sent = false;
                }
            }
            Message::ChooseFiles => {
                tasks.push(task::future(async move {
                    let file_filter = FileFilter::new("Any").glob("*.*");
                    let dialog = file_chooser::open::Dialog::new()
                        .title("Choose a file or files...")
                        .filter(file_filter);

                    let msg = match dialog.open_files().await {
                        Ok(file_responses) => {
                            Message::FilesSelected(file_responses.urls().to_vec())
                        }
                        Err(file_chooser::Error::Cancelled) => Message::FileChoosingCancelled,
                        Err(e) => {
                            eprintln!("Choosing a file or files went wrong: {e}");
                            Message::FileChoosingCancelled
                        }
                    };

                    msg
                }));
            }
            Message::FilesSelected(urls) => {
                for url in urls.iter() {
                    if let Ok(path) = url.to_file_path() {
                        if path.exists() {
                            if let Some(path) = path.to_str() {
                                self.send_files.push(path.to_string());
                            }
                        }
                    }
                    self.files_sent = false;
                    return self.reopen_popup();
                }
            }
            Message::SendFiles => {
                if self.selected_device_name != fl!("select-default") && !self.send_files.is_empty()
                {
                    self.files_sent = true;
                    let client = self.client.clone();
                    let files = self.send_files.clone();
                    let dev_name = self.selected_device_name.clone();
                    let notify = self.preferences.notifications_enabled;

                    // Find the peer ID for the selected device
                    let peer_id = self
                        .state
                        .devices
                        .iter()
                        .find(|dev| dev.name == dev_name)
                        .map(|dev| dev.id.clone())
                        .unwrap_or_default();

                    let file_count = files.len();

                    tasks.push(task::future(async move {
                        let result = send_files(&client, &peer_id, &files).await;
                        if notify && result.is_none() {
                            notify_files_sent(&dev_name, file_count);
                        }
                        Message::FilesSent(result)
                    }));
                }
            }
            Message::FilesSent(tx_status) => {
                // Once the files are sent:
                // 1. Set the send file status to the transfer status
                self.send_file_status = match tx_status {
                    Some(err_val) => err_val,
                    None => fl!("taildrop-files-sent"),
                };

                if !self.send_file_status.is_empty() {
                    if !self.send_files.is_empty() {
                        // 2. Clear the selected files that were just sent from the vector
                        self.send_files.clear();
                    }

                    // Create a task in a separate thread that clears the TailDrop status after a designated amount of time.
                    tasks.push(task::future(async move { Message::ClearTailDropStatus }));
                }
            }
            Message::FileChoosingCancelled => {
                return self.reopen_popup();
            }
            Message::RecieveFiles => {
                let client = self.client.clone();
                let download_dir = self
                    .preferences
                    .download_dir
                    .clone()
                    .unwrap_or_else(default_download_dir);

                let notify = self.preferences.notifications_enabled
                    && self.preferences.notify_on_incoming_files;

                tasks.push(task::future(async move {
                    match receive_files(&client, &download_dir).await {
                        Ok(names) => {
                            if notify {
                                notify_files_received(&download_dir);
                            }
                            Message::FilesRecieved(format!(
                                "Received {} file(s) in {download_dir}",
                                names.len()
                            ))
                        }
                        Err(e) => Message::FilesRecieved(e),
                    }
                }));
            }
            Message::FilesRecieved(rx_status) => {
                self.receive_file_status = rx_status;

                if !self.receive_file_status.is_empty() {
                    // The IPN bus signals new TailDrop files but not inbox
                    // clears, so refetch state to update waiting_files.
                    tasks.push(task::future(async move { Message::IpnEvent }));
                    // Clear the TailDrop status after a delay.
                    tasks.push(task::future(async move { Message::ClearTailDropStatus }));
                }
            }
            Message::ClearTailDropStatus => {
                if !self.receive_file_status.is_empty() {
                    tasks.push(task::future(async move {
                        clear_status(STATUS_CLEAR_TIME).await;
                        Message::FilesRecieved(String::new())
                    }));
                } else if !self.send_file_status.is_empty() || self.files_sent {
                    self.selected_device_idx = Some(0);
                    self.selected_device_name = fl!("select-default");
                    tasks.push(task::future(async move {
                        clear_status(STATUS_CLEAR_TIME).await;
                        Message::FilesRecieved(String::new())
                    }));
                }
            }
            Message::ExitNodeSelected(exit_node) => {
                if !self.state.is_exit_node {
                    self.sel_exit_node_idx = Some(exit_node);
                    let client = self.client.clone();

                    let node_ip = if exit_node == 0 {
                        String::new()
                    } else {
                        self.state
                            .exit_node_options
                            .get(exit_node - 1)
                            .and_then(|dev| dev.tailscale_ips.first())
                            .cloned()
                            .unwrap_or_default()
                    };

                    update_config(self.config.clone(), "exit-node", exit_node);

                    tasks.push(task::future(async move {
                        let _ = set_exit_node(&client, &node_ip).await;
                        Message::ActionCompleted(Ok(()))
                    }));
                }
            }
            Message::AllowExitNodeLanAccess(allow) => {
                if self.state.is_exit_node {
                    let client = self.client.clone();
                    update_config(self.config.clone(), "allow-lan", allow);
                    tasks.push(task::future(async move {
                        let _ = set_exit_node_allow_lan(&client, allow).await;
                        Message::ActionCompleted(Ok(()))
                    }));
                }
            }
            Message::UpdateIsExitNode(enable) => {
                if self.sel_exit_node_idx == Some(0) || self.sel_exit_node_idx.is_none() {
                    let client = self.client.clone();
                    tasks.push(task::future(async move {
                        let _ = set_advertise_exit_node(client.clone(), enable).await;
                        Message::ActionCompleted(Ok(()))
                    }));
                }
            }
            Message::SelectDeviceDetail(idx) => {
                self.selected_device_detail_idx = if self.selected_device_detail_idx == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
                self.ping_result = None;
            }
            Message::PingDevice(ip) => {
                self.ping_in_progress = true;
                let client = self.client.clone();
                tasks.push(task::future(async move {
                    match ping_device(&client, &ip).await {
                        Ok(ping_reply) => Message::PingCompleted(Ok(ping_reply)),
                        Err(e) => Message::PingCompleted(Err(e.to_string())),
                    }
                }));
            }
            Message::PingCompleted(result) => {
                self.ping_in_progress = false;
                self.ping_result = result.ok();
            }
            Message::CopyToClipboard(val) => {
                let _ = copy_to_clipboard(&val);
            }
            Message::SubnetInput(val) => {
                self.subnet_input = val;
            }
            Message::AddSubnet => {
                if !self.subnet_input.is_empty() {
                    let mut routes = self.state.advertised_routes.clone();
                    routes.push(self.subnet_input.clone());
                    self.subnet_input.clear();
                    let client = self.client.clone();
                    tasks.push(task::future(async move {
                        let _ = set_advertised_routes(&client, routes).await;
                        Message::ActionCompleted(Ok(()))
                    }));
                }
            }
            Message::RemoveSubnet(idx) => {
                if idx < self.state.advertised_routes.len() {
                    let mut routes = self.state.advertised_routes.clone();
                    routes.remove(idx);
                    let client = self.client.clone();
                    tasks.push(task::future(async move {
                        let _ = set_advertised_routes(&client, routes).await;
                        Message::ActionCompleted(Ok(()))
                    }));
                }
            }
            Message::SetAutoConnect(val) => {
                self.preferences.auto_connect = val;
                update_config(self.config.clone(), "auto-connect", val);
            }
            Message::SetNotificationsEnabled(val) => {
                self.preferences.notifications_enabled = val;
                update_config(self.config.clone(), "notifications-enabled", val);
            }
            Message::SetNotifyConnection(val) => {
                self.preferences.notify_on_connection_change = val;
                update_config(self.config.clone(), "notify-connection", val);
            }
            Message::SetNotifyFiles(val) => {
                self.preferences.notify_on_incoming_files = val;
                update_config(self.config.clone(), "notify-files", val);
            }
            Message::SetNotifyDevice(val) => {
                self.preferences.notify_on_new_device = val;
                update_config(self.config.clone(), "notify-device", val);
            }
            Message::SetIconStyle(dynamic) => {
                self.preferences.icon_style = if dynamic {
                    "dynamic".to_string()
                } else {
                    "static".to_string()
                };
                update_config(
                    self.config.clone(),
                    "icon-style",
                    self.preferences.icon_style.clone(),
                );
            }
            Message::ChooseDownloadDir => {
                tasks.push(task::future(async move {
                    let title = fl!("dir-chooser-title");
                    let dialog = file_chooser::open::Dialog::new().title(title);
                    match dialog.open_folders().await {
                        Ok(response) => Message::DownloadDirSelected(response.urls().to_vec()),
                        Err(_) => Message::DownloadDirCancelled,
                    }
                }));
            }
            Message::DownloadDirSelected(urls) => {
                if let Some(path) = urls
                    .into_iter()
                    .next()
                    .and_then(|url| url.to_file_path().ok())
                    .and_then(|p| p.to_str().map(str::to_string))
                {
                    self.preferences.download_dir = Some(path.clone());
                    update_config(self.config.clone(), "download-dir", path);
                }
                return self.reopen_popup();
            }
            Message::DownloadDirCancelled => return self.reopen_popup(),
            Message::ActionCompleted(result) => {
                if let Err(e) = result {
                    eprintln!("Tailscale action failed: {e}");
                }
            }
        }
        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    // Libcosmic's view function
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            // Set the icon button to the Tailscale icon (labeled as flatpak name) defined during installation.
            .icon_button("com.bhh32.gui-scale-applet")
            .on_press(Message::TogglePopup)
            .into()
    }

    // Libcosmic's applet view_window function
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
<<<<<<< HEAD
        let tab_bar = row![
            tab_button("network-vpn-symbolic", Tab::Status, self.active_tab),
            tab_button("send-to-symbolic", Tab::TailDrop, self.active_tab),
            tab_button("computer-symbolic", Tab::Devices, self.active_tab),
            tab_button(
                "preferences-system-symbolic",
                Tab::Settings,
                self.active_tab
            ),
        ]
        .spacing(4)
        .padding(4);

        let content: Element<'_, Message> = match self.active_tab {
            Tab::Status => self.view_status_tab(),
            Tab::TailDrop => self.view_taildrop_tab(),
            Tab::Devices => self.view_devices_tab(),
            Tab::Settings => self.view_settings_tab(),
        };
=======
        // Normal status elements
        let cur_acct = &self.cur_acct;
        let acct_list = &self.acct_list;
        let ip = get_tailscale_ip();

        // Get the current account index
        let mut sel_acct_idx = None;
        for (idx, acct) in acct_list.iter().enumerate() {
            if acct == cur_acct {
                sel_acct_idx = Some(idx);
                break;
            }
        }

        let conn_status = get_tailscale_con_status();

        let status_elements: Vec<Element<'_, Message>> = vec![
            (Element::from(column!(
                row!(settings::item(
                    "Account",
                    dropdown(acct_list, sel_acct_idx, Message::SwitchAccount)
                )),
                row!(settings::item("Tailscale Address", text(ip.clone()),)),
                row!(settings::item(
                    "Connection Status",
                    text(if conn_status {
                        "Tailscale Connected"
                    } else {
                        "Tailscale Disconnected"
                    })
                )),
            ))),
        ];
>>>>>>> a2689dd (Add account switching)

        let body = column![tab_bar, scrollable(content)].spacing(4);
        self.core.applet.popup_container(body).into()
    }
}

fn detail_row(label: String, value: String) -> Element<'static, Message> {
    row![
        text(label).size(11).width(Length::Fixed(80.0)),
        text(value).size(11),
    ]
    .spacing(8)
    .into()
}

fn tab_button(icon_name: &'static str, tab: Tab, active: Tab) -> Element<'static, Message> {
    let btn = button::icon(icon::from_name(icon_name)).on_press(Message::TabSelected(tab));
    let btn = if tab == active {
        btn.class(cosmic::theme::Button::Suggested)
    } else {
        btn
    };
    btn.into()
}

impl Window {
    fn reopen_popup(&mut self) -> Task<Action<Message>> {
        let new_id = Id::unique();
        self.popup.replace(new_id);
        let mut popup_settings = self.core.applet.get_popup_settings(
            self.core.main_window_id().unwrap(),
            new_id,
            None,
            None,
            None,
        );
        popup_settings.positioner.size_limits = Limits::NONE
            .max_width(POPUP_MAX_WIDTH)
            .min_width(POPUP_MIN_WIDTH)
            .max_height(POPUP_MAX_HEIGHT)
            .min_height(POPUP_MIN_HEIGHT);
        get_popup(popup_settings)
    }

    fn view_status_tab(&self) -> Element<'_, Message> {
        let state = &self.state;
        let sel_acct_idx = state.accounts.iter().position(|acct| acct.is_current);

        let conn_status = if state.connected {
            fl!("status-connected")
        } else {
            fl!("status-disconnected")
        };

        let status_elements = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                fl!("status-account"),
                column![
                    container(
                        button::standard(fl!("status-new-login"))
                            .on_press(Message::LoginNewAccount)
                    ),
                    dropdown(&self.acct_names, sel_acct_idx, Message::SwitchAccount),
                ]
                .spacing(8)
                .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-ipv4"),
                row![
                    text(&state.ip_v4),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(state.ip_v4.clone()))
                        .tooltip(fl!("copy-tooltip")),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-ipv6"),
                row![
                    text(&state.ip_v6),
                    button::icon(icon::from_name("edit-copy-symbolic"))
                        .on_press(Message::CopyToClipboard(state.ip_v6.clone()))
                        .tooltip(fl!("copy-tooltip")),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            ))
            .add(settings::item(fl!("status-connection"), text(conn_status)))
            .add(settings::item(
                fl!("status-enable-ssh"),
                container(toggler(state.ssh_enabled).on_toggle(Message::EnableSSH))
                    .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-accept-routes"),
                container(toggler(state.accept_routes).on_toggle(Message::AcceptRoutes))
                    .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-magic-dns"),
                container(toggler(state.magic_dns).on_toggle(Message::ToggleMagicDns))
                    .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-magic-dns-suffix"),
                container(text(if state.dns_suffix.is_empty() {
                    "—".to_string()
                } else {
                    state.dns_suffix.clone()
                }))
                .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-advertise-exit"),
                container(toggler(state.is_exit_node).on_toggle(Message::UpdateIsExitNode))
                    .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-allow-lan-access"),
                container(
                    toggler(state.exit_node_allow_lan).on_toggle(Message::AllowExitNodeLanAccess),
                )
                .align_x(Alignment::End),
            ))
            .add(settings::item(
                fl!("status-connect-toggle"),
                container(toggler(state.connected).on_toggle(Message::ConnectDisconnect))
                    .align_x(Alignment::End),
            ));

        // Subnet routes section
        let mut subnets_section = column![text(fl!("subnets-title")).size(14)]
            .spacing(4)
            .padding(4);

        if state.advertised_routes.is_empty() {
            subnets_section = subnets_section.push(text(fl!("subnets-no-routes")).size(12));
        } else {
            for (idx, route) in state.advertised_routes.iter().enumerate() {
                subnets_section = subnets_section.push(
                    row![
                        text(route).width(Length::Fill),
                        button::destructive(fl!("subnets-remove"))
                            .on_press(Message::RemoveSubnet(idx))
                            .width(Length::Shrink),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }

        subnets_section = subnets_section.push(
            row![
                text_input(fl!("subnets-cidr-placeholder"), &self.subnet_input)
                    .on_input(Message::SubnetInput)
                    .width(250),
                button::suggested(fl!("subnets-add"))
                    .on_press(Message::AddSubnet)
                    .width(Length::Shrink),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        );

        Element::from(
            column![
                Element::from(status_elements),
                Element::from(subnets_section)
            ]
            .width(1024)
            .spacing(8),
        )
    }

    fn view_taildrop_tab(&self) -> Element<'_, Message> {
        // --- Send section ---
        let send_header = text(fl!("taildrop-send-title")).size(14);

        let device_picker = row![
            text(fl!("taildrop-send-to")).width(Length::Shrink),
            dropdown(
                &self.state.device_names,
                self.selected_device_idx,
                Message::DeviceSelected,
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let mut send_section = column![send_header, device_picker].spacing(6).padding(4);

        if !self.send_files.is_empty() {
            let mut files_col = column![text(fl!("taildrop-files-queued")).size(12)].spacing(2);
            for path in &self.send_files {
                files_col = files_col.push(text(path.clone()).size(11));
            }
            send_section = send_section.push(files_col);
        }

        let send_buttons = row![
            button::standard(fl!("taildrop-choose-files")).on_press(Message::ChooseFiles),
            button::suggested(fl!("taildrop-send")).on_press(Message::SendFiles),
        ]
        .spacing(8);
        send_section = send_section.push(send_buttons);

        if !self.send_file_status.is_empty() {
            send_section = send_section.push(text(self.send_file_status.clone()).size(11));
        }

        // --- Receive section ---
        let recv_header = text(fl!("taildrop-receive-title")).size(14);
        let mut recv_section = column![recv_header].spacing(6).padding(4);

        if self.state.waiting_files.is_empty() {
            recv_section = recv_section.push(text(fl!("taildrop-no-incoming")).size(12));
        } else {
            for f in &self.state.waiting_files {
                let line = format!("{} ({})", f.name, format_bytes(f.size));
                recv_section = recv_section.push(text(line).size(11));
            }
            recv_section = recv_section
                .push(button::suggested(fl!("taildrop-receive")).on_press(Message::RecieveFiles));
        }

        if !self.receive_file_status.is_empty() {
            recv_section = recv_section.push(text(self.receive_file_status.clone()).size(11));
        }

        column![send_section, recv_section]
            .spacing(10)
            .padding(4)
            .into()
    }

    fn view_devices_tab(&self) -> Element<'_, Message> {
        let header = text(fl!("devices-title")).size(14);
        let mut col = column![header].spacing(4).padding(4);

        if self.state.devices.is_empty() {
            col = col.push(text(fl!("devices-none")).size(12));
            return col.into();
        }

        for (idx, dev) in self.state.devices.iter().enumerate() {
            let expanded = self.selected_device_detail_idx == Some(idx);
            let dot = if dev.online { "● " } else { "○ " };
            let label = if dev.is_self {
                format!("{dot}{} ({})", dev.name, fl!("devices-self"))
            } else {
                format!("{dot}{}", dev.name)
            };

            let header_btn = button::text(label)
                .on_press(Message::SelectDeviceDetail(idx))
                .width(Length::Fill);
            col = col.push(header_btn);

            if expanded {
                let mut detail = column![].spacing(3).padding([0, 0, 4, 16]);

                if let Some(ipv4) = dev.tailscale_ips.first() {
                    detail = detail.push(detail_row(fl!("status-ipv4"), ipv4.clone()));
                }
                if let Some(ipv6) = dev.tailscale_ips.get(1) {
                    detail = detail.push(detail_row(fl!("status-ipv6"), ipv6.clone()));
                }
                if !dev.os.is_empty() {
                    detail = detail.push(detail_row(fl!("devices-os"), dev.os.clone()));
                }
                if !dev.dns_name.is_empty() {
                    detail = detail.push(detail_row(fl!("devices-dns"), dev.dns_name.clone()));
                }
                if !dev.relay.is_empty() {
                    detail = detail.push(detail_row(fl!("devices-relay"), dev.relay.clone()));
                }
                if !dev.tags.is_empty() {
                    detail = detail.push(detail_row(fl!("devices-tags"), dev.tags.join(", ")));
                }
                detail = detail.push(detail_row(fl!("devices-rx"), format_bytes(dev.rx_bytes)));
                detail = detail.push(detail_row(fl!("devices-tx"), format_bytes(dev.tx_bytes)));
                if !dev.online && !dev.last_seen.is_empty() {
                    detail =
                        detail.push(detail_row(fl!("devices-last-seen"), dev.last_seen.clone()));
                }

                let mut actions = row![].spacing(8);
                if !dev.is_self {
                    if let Some(ip) = dev.tailscale_ips.first() {
                        actions = actions.push(
                            button::standard(fl!("devices-ping"))
                                .on_press(Message::PingDevice(ip.clone())),
                        );
                    }
                }
                if let Some(ip) = dev.tailscale_ips.first() {
                    actions = actions.push(
                        button::standard(fl!("devices-copy-ip"))
                            .on_press(Message::CopyToClipboard(ip.clone())),
                    );
                }
                if dev.exit_node_option && !dev.is_self {
                    let label = if dev.is_exit_node {
                        fl!("devices-exit-current")
                    } else {
                        fl!("devices-use-as-exit")
                    };
                    let target_idx = self
                        .exit_node_names
                        .iter()
                        .position(|n| n == &dev.name)
                        .unwrap_or(0);
                    actions = actions.push(
                        button::standard(label).on_press(Message::ExitNodeSelected(target_idx)),
                    );
                }
                detail = detail.push(actions);

                if self.ping_in_progress {
                    detail = detail.push(text(fl!("devices-pinging")).size(11));
                } else if let Some(ref pr) = self.ping_result {
                    let line = if !pr.err.is_empty() {
                        format!("{}: {}", fl!("devices-ping-error"), pr.err)
                    } else {
                        format!(
                            "{}: {:.1} ms ({})",
                            fl!("devices-ping-result"),
                            pr.latency_seconds * 1000.0,
                            if pr.is_direct {
                                fl!("devices-direct")
                            } else {
                                fl!("devices-relayed")
                            }
                        )
                    };
                    detail = detail.push(text(line).size(11));
                }

                col = col.push(detail);
            }
        }

        col.into()
    }

    fn view_settings_tab(&self) -> Element<'_, Message> {
        let prefs = &self.preferences;
        let download_dir = prefs
            .download_dir
            .clone()
            .unwrap_or_else(default_download_dir);

        let elements = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                fl!("settings-auto-connect"),
                toggler(prefs.auto_connect).on_toggle(Message::SetAutoConnect),
            ))
            .add(settings::item(
                fl!("settings-icon-dynamic"),
                toggler(prefs.icon_style == "dynamic").on_toggle(Message::SetIconStyle),
            ))
            .add(settings::item(
                fl!("settings-notifications"),
                toggler(prefs.notifications_enabled).on_toggle(Message::SetNotificationsEnabled),
            ))
            .add(settings::item(
                fl!("settings-notify-connection"),
                toggler(prefs.notify_on_connection_change).on_toggle(Message::SetNotifyConnection),
            ))
            .add(settings::item(
                fl!("settings-notify-files"),
                toggler(prefs.notify_on_incoming_files).on_toggle(Message::SetNotifyFiles),
            ))
            .add(settings::item(
                fl!("settings-notify-device"),
                toggler(prefs.notify_on_new_device).on_toggle(Message::SetNotifyDevice),
            ))
            .add(settings::item(
                fl!("settings-download-dir"),
                row![column![
                    text(download_dir)
                        .width(Length::Fill)
                        .align_x(Alignment::End),
                    container(
                        button::standard(fl!("settings-change"))
                            .on_press(Message::ChooseDownloadDir)
                    )
                    .width(Length::Fill)
                    .padding(4)
                    .align_x(Alignment::End)
                ]]
                .spacing(8)
                .width(1024.0)
                .align_y(Alignment::Center),
            ));

        column![elements].padding(4).into()
    }
}
