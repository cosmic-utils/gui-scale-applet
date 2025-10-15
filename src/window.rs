use crate::config::{load_config, update_config};
use crate::logic::{
    clear_status, enable_exit_node, exit_node_allow_lan_access, get_avail_exit_nodes,
    get_is_exit_node, get_tailscale_con_status, get_tailscale_devices, get_tailscale_ip,
    get_tailscale_routes_status, get_tailscale_ssh_status, set_exit_node, set_routes, set_ssh,
    tailscale_int_up, tailscale_recieve, tailscale_send,
};
use cosmic::app::Core;
use cosmic::cosmic_config::Config;
use cosmic::dialog::file_chooser::{self, FileFilter};
use cosmic::iced::{
    alignment::Horizontal,
    platform_specific::shell::commands::popup::{destroy_popup, get_popup},
    widget::{column, horizontal_space, row},
    window::Id,
    Alignment, Length, Limits,
};
use cosmic::iced_runtime::core::window;
use cosmic::iced_widget::Row;
use cosmic::widget::{
    button, dropdown, list_column,
    settings::{self},
    text, toggler,
};
use cosmic::{Action, Element, Task};
use std::fmt::Debug;
use std::path::PathBuf;
use url::Url;

const ID: &str = "com.github.bhh32.GUIScaleApplet";
const CONFIG_VERS: u64 = 1;
const DEFAULT_EXIT_NODE: &str = "Select Exit Node";
const POPUP_MAX_WIDTH: f32 = 400.0;
const POPUP_MIN_WIDTH: f32 = 300.0;
const POPUP_MAX_HEIGHT: f32 = 1080.0;
const POPUP_MIN_HEIGHT: f32 = 200.0;
const STATUS_CLEAR_TIME: u64 = 5;

/// Holds the applet's state
pub struct Window {
    core: Core,
    config: Config,
    popup: Option<Id>,
    ssh: bool,
    routes: bool,
    connect: bool,
    device_options: Vec<String>,
    selected_device: String,
    selected_device_idx: Option<usize>,
    send_files: Vec<Option<String>>,
    send_file_status: String,
    files_sent: bool,
    recieve_file_status: String,
    avail_exit_nodes: Vec<String>,
    sel_exit_node: String,
    sel_exit_node_idx: Option<usize>,
    allow_lan: bool,
    is_exit_node: bool,
}

/// Messages to be sent to the Libcosmic Update function
#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    EnableSSH(bool),
    AcceptRoutes(bool),
    ConnectDisconnect(bool),
    DeviceSelected(usize),
    ChooseFiles,
    FilesSelected(Vec<Url>),
    SendFiles,
    FilesSent(Option<String>),
    FileChoosingCancelled,
    RecieveFiles,
    FilesRecieved(String),
    ExitNodeSelected(usize),
    AllowExitNodeLanAccess(bool),
    UpdateIsExitNode(bool),
    ClearTailDropStatus,
}

impl cosmic::Application for Window {
    type Executor = cosmic::executor::multi::Executor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Window, Task<Action<Self::Message>>) {
        // Get the SSH status from the Tailscale CLI
        let ssh = get_tailscale_ssh_status();
        // Get the Accept Routes status from the Tailscale CLI
        let routes = get_tailscale_routes_status();
        // Get the connection status from the Tailscale CLI
        let connect = get_tailscale_con_status();
        // Get the other devices on the Tailnet from the Tailscale CLI
        let dev_init = get_tailscale_devices();

        // Set the default applet state for allow_lan to false
        let allow_lan = false;
        // Get the state of the host being an exit node from the Tailscale CLI
        let is_exit_node = get_is_exit_node();

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

        // Set the start up state of the application using the above variables
        let mut window = Window {
            core,
            config: Config::new(ID, CONFIG_VERS).unwrap(),
            ssh,
            routes,
            connect,
            device_options: dev_init,
            popup: None,
            selected_device: DEFAULT_EXIT_NODE.to_string(),
            selected_device_idx: Some(0),
            send_files: Vec::<Option<String>>::new(),
            send_file_status: String::new(),
            files_sent: false,
            recieve_file_status: String::new(),
            avail_exit_nodes: exit_nodes_init,
            sel_exit_node: DEFAULT_EXIT_NODE.to_string(),
            sel_exit_node_idx: None,
            allow_lan,
            is_exit_node,
        };

        // Set the exit node index state from the config file
        window.sel_exit_node_idx = match load_config("exit-node", CONFIG_VERS) {
            (Some(val), _) => Some(val),
            (None, err_str) => {
                eprintln!("{err_str}");
                None
            }
        };

        // Set the allow lan state from the config file
        window.allow_lan = match load_config("allow-lan", CONFIG_VERS) {
            (Some(val), _) => val,
            (None, err_str) => {
                eprintln!("{err_str}");
                false
            }
        };

        // Return the state and no Task
        (window, Task::none())
    }

    // The function that is called when the applet is closed
    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    // Libcosmic's update function
    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    self.recieve_file_status = String::new();
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
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::EnableSSH(enabled) => {
                self.ssh = enabled;
                set_ssh(self.ssh);
            }
            Message::AcceptRoutes(accepted) => {
                self.routes = accepted;
                set_routes(self.routes);
            }
            Message::ConnectDisconnect(connection) => {
                self.connect = connection;
                tailscale_int_up(self.connect);
            }
            Message::DeviceSelected(device) => {
                self.selected_device = self.device_options[device].clone();
                self.selected_device_idx = Some(device);

                if self.files_sent {
                    self.files_sent = false;
                }
            }
            Message::ChooseFiles => {
                return cosmic::task::future(async move {
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
                });
            }
            Message::FilesSelected(urls) => {
                for url in urls.iter() {
                    let path = match url.to_file_path() {
                        Ok(good_path) => good_path,
                        Err(_e) => PathBuf::new(),
                    };

                    if path.exists() {
                        self.send_files.push(Some(match path.as_path().to_str() {
                            Some(f_path) => String::from(f_path),
                            None => String::new(),
                        }));
                    }
                }

                // Set the files sent flag to false.
                self.files_sent = false;

                // Use the same popup logic as TogglePopup to keep the applet open
                // after selecting the files.
                // Note: It won't let you just call Message::TogglePopup here.
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

                return get_popup(popup_settings);
            }
            Message::SendFiles => {
                // Get the file(s) that are being sent
                let files = self.send_files.clone();
                // Get the device name that the files are being sent to
                let dev = self.selected_device.clone();

                // Make sure that the device is not the Select choice
                if dev != "Select" {
                    self.files_sent = true;
                    // Use the async command to use a new thread
                    return cosmic::task::future(async move {
                        // Send the file(s) and return the transfer status when the transfer is complete

                        // Status clearing bug starts here. Unsure why this doesn't wait for the status to return before
                        // sending the FilesSent message.
                        let tx_status = (tailscale_send(files, &dev)).await;

                        // When the file(s) are done being sent, send the FilesSent message to the update function
                        Message::FilesSent(tx_status)
                    });
                }
            }
            Message::FilesSent(tx_status) => {
                println!("tx_status: {tx_status:?}");
                // Once the files are sent:
                // 1. Set the send file status to the transfer status
                self.send_file_status = match tx_status {
                    Some(err_val) => err_val,
                    None => String::from("File(s) sent successfully!"),
                };

                if !self.send_file_status.is_empty() {
                    if !self.send_files.is_empty() {
                        // 2. Clear the selected files that were just sent from the vector
                        self.send_files.clear();
                    }

                    // Create a task in a separate thread that clears the TailDrop status after a designated amount of time.
                    return cosmic::task::future(async move { Message::ClearTailDropStatus });
                }
            }
            Message::FileChoosingCancelled => {
                // Use the same popup logic as TogglePopup to keep the applet open
                // after selecting the files.
                // Note: It won't let you just call Message::TogglePopup here.
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

                return get_popup(popup_settings);
            }
            Message::RecieveFiles => {
                // Run the recieve function in a separate thread so it doesn't block the current thread.
                return cosmic::task::future(async move {
                    let rx_status = tailscale_recieve().await;
                    Message::FilesRecieved(rx_status)
                });
            }
            Message::FilesRecieved(rx_status) => {
                self.recieve_file_status = rx_status;

                if !self.recieve_file_status.is_empty() {
                    // Create a task in a separate thread that clears the TailDrop status after a designated amount of time.
                    return cosmic::task::future(async move { Message::ClearTailDropStatus });
                }
            }
            Message::ExitNodeSelected(exit_node) => {
                if !self.is_exit_node {
                    // Set the model's selected exit node
                    self.sel_exit_node = self.avail_exit_nodes[exit_node].clone();
                    self.sel_exit_node_idx = Some(exit_node);

                    // Use that exit node
                    if exit_node == 0 {
                        set_exit_node(String::new());
                    } else {
                        set_exit_node(self.sel_exit_node.clone());
                    }

                    // Set the config_entry to the exit node
                    update_config(
                        self.config.clone(),
                        "exit-node",
                        match self.sel_exit_node_idx {
                            Some(idx) => idx,
                            None => {
                                eprintln!("Could not update the config file!");
                                0
                            }
                        },
                    );
                }
            }
            Message::AllowExitNodeLanAccess(allow_lan_access) => {
                self.allow_lan = allow_lan_access;

                // Double check that is_exit_node is true
                if self.is_exit_node {
                    // Set the host exit node to allow lan access
                    let _status = exit_node_allow_lan_access(self.allow_lan);
                    // Update the configuration file, allow-lan
                    update_config(self.config.clone(), "allow-lan", self.allow_lan);
                }
            }
            Message::UpdateIsExitNode(is_exit_node) => {
                // Ensure we're not using some other exit node
                if self.sel_exit_node_idx == Some(0) {
                    // Set the model is_exit_node to the message is_exit_node
                    self.is_exit_node = is_exit_node;

                    // Enable/disable this host as an exit node
                    enable_exit_node(self.is_exit_node);

                    self.avail_exit_nodes = get_avail_exit_nodes();
                }
            }
            Message::ClearTailDropStatus => {
                // Clear the files recieved status in the status clear time
                if !self.recieve_file_status.is_empty() {
                    // Done in a separate thread as to not block the current thread.
                    return cosmic::task::future(async move {
                        Message::FilesRecieved(match clear_status(STATUS_CLEAR_TIME).await {
                            Some(bad_value) => format!("Something went wrong and clear status returned a value: {bad_value}"),
                            None => String::new(),
                        })
                    });
                // Clear the send files status in the status clear time
                } else if !self.send_file_status.is_empty() || self.files_sent {
                    println!("Entered clear tail drop status message send");
                    // 4. Reset the selected_device_idx back to 0 (Selected)
                    self.selected_device_idx = Some(0);
                    // 5. Reset the selected_device back to Selected
                    self.selected_device = self.device_options[0].clone();

                    // Done in a separate thread as to not block the current thread.
                    return cosmic::task::future(async move {
                        Message::FilesSent(match clear_status(STATUS_CLEAR_TIME).await {
                            Some(bad_value) => Some(format!("Something went wrong and clear status returned a value: {bad_value}")),
                            None => Some(String::new()),
                        })
                    });
                }
            }
        }
        Task::none()
    }

    // Libcosmic's view function
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            // Set the icon button to the tailscale-icon defined during installation.
            .icon_button("tailscale-icon")
            .on_press(Message::TogglePopup)
            .into()
    }

    // Libcosmic's applet view_window function
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        // Normal status elements
        let ip = get_tailscale_ip();
        let conn_status = get_tailscale_con_status();

        let status_elements: Vec<Element<'_, Message>> = vec![
            (Element::from(column!(
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

        let status_row = Row::with_children(status_elements)
            .align_y(Alignment::Center)
            .spacing(0);

        // Enable/Disable Elements (ssh, routes)
        let enable_elements: Vec<Element<'_, Message>> = vec![
            (Element::from(
                column!(
                    row!(settings::item(
                        "Enable SSH",
                        toggler(self.ssh).on_toggle(Message::EnableSSH)
                    )),
                    row!(settings::item(
                        "Accept Routes",
                        toggler(self.routes).on_toggle(Message::AcceptRoutes)
                    )),
                )
                .spacing(5),
            )),
        ];

        let enable_row = Row::with_children(enable_elements);

        // File tx/rx elements
        let taildrop_elements: Vec<Element<'_, Message>> = vec![Element::from(
            column!(
                row!(text("Tail Drop")).align_y(Alignment::Center),
                row!(
                    column!(dropdown(
                        &self.device_options,
                        self.selected_device_idx,
                        Message::DeviceSelected
                    )
                    .width(140),)
                    .align_x(Horizontal::Left)
                    .padding(5),
                    horizontal_space().width(100),
                    column!(button::standard("Select File(s)")
                        .on_press(Message::ChooseFiles)
                        .width(140)
                        .tooltip("Select the file(s) to send."))
                    .align_x(Horizontal::Right)
                    .padding(5)
                )
                .align_y(Alignment::Center)
                .spacing(25),
                row!(
                    column!(if !self.send_files.is_empty() {
                        button::standard("Send File(s)")
                            .on_press(Message::SendFiles)
                            .width(140)
                            .tooltip("Send the selected file(s).")
                    } else {
                        button::standard("Send File(s)")
                            .width(140)
                            .tooltip("Send the selected file(s).")
                    })
                    .align_x(Horizontal::Left)
                    .padding(5),
                    horizontal_space().width(100),
                    column!(button::standard("Recieve File(s)")
                        .on_press(Message::RecieveFiles)
                        .width(140)
                        .tooltip("Recieve files waiting in the Tail Drop inbox."))
                    .align_x(Horizontal::Right)
                    .padding(5)
                )
                .align_y(Alignment::Center)
                .spacing(25)
            )
            .align_x(Alignment::Center),
        )];

        let taildrop_row = Row::with_children(taildrop_elements);
        // File tx/rx status elements
        let taildrop_status_elements: Vec<Element<'_, Message>> = vec![
            (Element::from(column!(
                row!(text("Send/Recieve Status")
                    .width(Length::Fill)
                    .align_x(Horizontal::Center))
                .height(30)
                .align_y(Alignment::Center),
                row!(if !self.send_file_status.is_empty() {
                    text(self.send_file_status.clone())
                } else if self.files_sent && self.selected_device != *"Select" {
                    text("File(s) were sent successfully!")
                } else if self.selected_device == *"Select" && !self.files_sent {
                    text("Choose a device first,\nthen reselect your file(s)!")
                } else {
                    text("")
                }),
                row!(text(self.recieve_file_status.clone()))
            ))),
        ];

        let tx_rx_status_row = Row::with_children(taildrop_status_elements);

        // Exit node UI elements
        // Using the config file to see if there is an external exit node set
        let (config_exit_node, _err): (Option<usize>, String) =
            load_config("exit-node", CONFIG_VERS);

        // Create element Vector for the exit node elements
        let mut exit_node_elements: Vec<Element<'_, Message>> = Vec::new();

        let host_exit_node_col = column!(
            Element::from(if config_exit_node == Some(0) {
                if !self.is_exit_node {
                    toggler(self.is_exit_node)
                        .label("Enable Host Exit Node")
                        .on_toggle(Message::UpdateIsExitNode)
                } else {
                    toggler(self.is_exit_node)
                        .label("Disable Host Exit Node")
                        .on_toggle(Message::UpdateIsExitNode)
                }
            } else {
                toggler(self.is_exit_node).label("Enable Host Exit Node")
            }),
            Element::from(if self.is_exit_node {
                toggler(self.allow_lan)
                    .label("Allow LAN Access")
                    .on_toggle(Message::AllowExitNodeLanAccess)
            } else {
                toggler(self.allow_lan).label("Allow LAN Access")
            })
        )
        .spacing(5)
        .align_x(Alignment::Start);

        exit_node_elements.push(Element::from(
            column!(
                row!(
                    // Section title
                    text("Exit Node")
                        .width(Length::Fill)
                        .align_x(Horizontal::Center)
                ),
                row!(
                    column!(
                        // Exit node selection dropdown
                        column!(
                            text("Selected Node")
                                .align_x(Alignment::Start)
                                .align_y(Alignment::Center),
                            dropdown(
                                &self.avail_exit_nodes,
                                self.sel_exit_node_idx,
                                Message::ExitNodeSelected
                            )
                            .width(125)
                        )
                        .align_x(Alignment::Center)
                    )
                    .padding(15)
                    .align_x(Alignment::Center),
                    column!(
                        // Use the config exit node setting to enable/disable the host's exit node toggler.
                        host_exit_node_col
                    )
                    .padding(15)
                )
            )
            .spacing(10)
            .align_x(Alignment::Center),
        ));

        let exit_node_row = Row::with_children(exit_node_elements);

        let content_list = list_column()
            .padding(5)
            .spacing(0)
            .add(Element::from(status_row))
            .add(Element::from(enable_row))
            .add(settings::item(
                "Connected",
                toggler(self.connect).on_toggle(Message::ConnectDisconnect),
            ))
            .add(Element::from(taildrop_row))
            .add(Element::from(tx_rx_status_row))
            .add(Element::from(exit_node_row));

        self.core.applet.popup_container(content_list).into()
    }
}
