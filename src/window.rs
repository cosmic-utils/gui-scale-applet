use std::path::PathBuf;
use cosmic::dialog::file_chooser::{self, FileFilter};
use cosmic::iced::alignment::Horizontal;
use cosmic::iced::Length;
use cosmic::app::Core;
use cosmic::iced::widget::{
    self, 
    column, 
    row,
    horizontal_space,
};
use cosmic::iced_widget::Row;
use cosmic::iced::{
    wayland::popup::{destroy_popup, get_popup},
    window::Id,
    Command, 
    Limits,
    Alignment,
};
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::widget::{list_column, settings, text, toggler, combo_box, button};
use cosmic::{Element, Theme};
use url::Url;
use crate::logic::{
    get_tailscale_con_status, 
    get_tailscale_devices, 
    get_tailscale_ip, 
    get_tailscale_routes_status, 
    get_tailscale_ssh_status, 
    set_routes, 
    set_ssh,
    tailscale_int_up, 
    tailscale_recieve, 
    tailscale_send
};

const ID: &str = "com.github.bhh32.GUIScaleApplet";
const DEFAULT_DEV: &str = "No Device";

pub struct Window {
    core: Core,
    popup: Option<Id>,
    ssh: bool,
    routes: bool,
    connect: bool,
    device_state: cosmic::widget::combo_box::State<String>,
    selected_device: String,
    send_files: Vec<Option<String>>,
    send_file_status: Option<String>,
    files_sent: bool,
    recieve_file_status: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    EnableSSH(bool),
    AcceptRoutes(bool),
    ConnectDisconnect(bool),
    DeviceSelected(String),
    ChooseFiles,
    FilesSelected(Vec<Url>),
    SendFiles,
    FilesSent(Option<String>),
    FileChoosingCancelled,
    RecieveFiles,
    FilesRecieved(String),
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

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, Command<cosmic::app::Message<Self::Message>>) {
        let ssh = get_tailscale_ssh_status();
        let routes = get_tailscale_routes_status();
        let connect = get_tailscale_con_status();
        let dev_init = get_tailscale_devices();
        let window = Window {
            core,
            ssh,
            routes,
            connect,
            device_state: widget::combo_box::State::new(dev_init),
            popup: None,
            selected_device: DEFAULT_DEV.to_string(),
            send_files: Vec::<Option<String>>::new(),
            send_file_status: None,
            files_sent: false,
            recieve_file_status: String::new(),
        };

        (window, Command::none())
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Command<cosmic::app::Message<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    self.recieve_file_status = String::new();
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);

                    let mut popup_settings =
                        self.core
                            .applet
                            .get_popup_settings(Id::MAIN, new_id, None, None, None);

                    popup_settings.positioner.size_limits = Limits::NONE
                            .max_width(372.0)
                            .min_width(300.0)
                            .min_height(200.0)
                            .max_height(1080.0);
                    
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
                self.selected_device = device;
                
                if self.files_sent {
                    self.files_sent = false;
                }
            }
            Message::ChooseFiles => {
                return cosmic::command::future(async move {
                    let file_filter = FileFilter::new("Any").glob("*.*");
                    let dialog = file_chooser::open::Dialog::new()
                        .title("Choose a file or files...")
                        .filter(file_filter);

                    let msg = match dialog.open_files().await {
                        Ok(file_responses) => Message::FilesSelected(file_responses.urls().to_vec()),
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
                            None => String::new()
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

                let mut popup_settings =
                    self.core
                        .applet
                        .get_popup_settings(Id::MAIN, new_id, None, None, None);

                popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    
                return get_popup(popup_settings);
            }
            Message::SendFiles => {
                let files = self.send_files.clone();
                let dev = self.selected_device.clone();

                return cosmic::command::future(async move {                    
                    let tx_status = tailscale_send(files, &dev).await;
                    Message::FilesSent(tx_status)
                });
                
            }
            Message::FilesSent(tx_status) => {
                self.send_file_status = tx_status;
                self.send_files.clear();
                self.files_sent = true;
            }
            Message::FileChoosingCancelled => {
                // Use the same popup logic as TogglePopup to keep the applet open
                // after selecting the files.
                // Note: It won't let you just call Message::TogglePopup here.
                let new_id = Id::unique();
                self.popup.replace(new_id);

                let mut popup_settings =
                    self.core
                        .applet
                        .get_popup_settings(Id::MAIN, new_id, None, None, None);

                popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    
                return get_popup(popup_settings);
            }
            Message::RecieveFiles => {
                return cosmic::command::future(async move {
                    let rx_status = tailscale_recieve().await;
                    Message::FilesRecieved(rx_status)
                });
            }
            Message::FilesRecieved(rx_status) => {
                self.recieve_file_status = rx_status;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        self.core
            .applet
            .icon_button("tailscale-icon")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        // Normal status elements
        let ip = get_tailscale_ip();
        let conn_status = get_tailscale_con_status();

        let mut status_elements: Vec<Element<'_, Message>> = Vec::new();
        status_elements.push(Element::from(
            column!(
                row!(
                settings::item(
                    "Tailscale Address",
                    text(ip.clone()),
                )),
                row!(settings::item(
                    "Connection Status",
                    text(if conn_status { "Tailscale Connected" } else { "Tailscale Disconnected" })
                )),
            )
        ));

        let status_row = Row::with_children(status_elements)
            .align_items(Alignment::Center)
            .spacing(0);

        // Enable/Disable Elements (ssh, routes)
        let mut enable_elements: Vec<Element<'_, Message>> = Vec::new();
        enable_elements.push(Element::from(
            column!(
                row!(settings::item(
                    "Enable SSH",
                    toggler(None, self.ssh, |value| {
                        Message::EnableSSH(value)
                    })
                )),
                row!(settings::item(
                    "Accept Routes",
                    toggler(None, self.routes, |value| {
                        Message::AcceptRoutes(value)
                    })
                )),
            )
            .spacing(5)
        ));

        let enable_row = Row::with_children(enable_elements);

        // File tx/rx elements
        let mut taildrop_elements: Vec<Element<'_, Message>> = Vec::new();
        taildrop_elements.push(Element::from(
            column!(
                row!().height(10).width(Length::Fill),
                row!(text("Tail Drop"))
                    .align_items(Alignment::Center),
                row!(
                    combo_box(&self.device_state, DEFAULT_DEV, Some(&self.selected_device), Message::DeviceSelected)
                        .width(140),
                    horizontal_space(Length::Fill),
                    button::standard("Select File(s)")
                        .on_press(Message::ChooseFiles)
                        .width(140)
                        .tooltip("Select the file(s) to send.")
                )
                .height(30),
                row!(
                    if self.send_files.len() > 0 {
                    button::standard("Send File(s)")
                        .on_press(Message::SendFiles)
                        .width(140)
                        .tooltip("Send the selected file(s).")
                    } else {
                        button::standard("Send File(s)")
                        .width(140)
                        .tooltip("Send the selected file(s).")
                    },
                    horizontal_space(Length::Fill),
                    button::standard("Recieve File(s)")
                        .on_press(Message::RecieveFiles)
                        .width(140)
                        .tooltip("Recieve files waiting in the Tail Drop inbox.")
                )
                .align_items(Alignment::Center)
                .height(30),
                row!().height(10).width(Length::Fill),
            )
            .spacing(5)
            .align_items(Alignment::Center)
        ));
        
        let taildrop_row = Row::with_children(taildrop_elements);

        // File tx/rx status elements
        let mut taildrop_status_elements: Vec<Element<'_, Message>> = Vec::new();
        taildrop_status_elements.push(Element::from(
            column!(
                row!(
                    text("Send/Recieve Status")
                        .width(Length::Fill)
                        .horizontal_alignment(Horizontal::Center)
                )
                .height(30)
                .align_items(Alignment::Center),
                row!(
                    match &self.send_file_status {
                        Some(tx_status) => text(tx_status.clone()),
                        None => {
                            if self.files_sent && self.selected_device != "No Device".to_string() {
                                text("File(s) were sent successfully!")
                            } else if self.files_sent && self.selected_device == "No Device".to_string() {
                                text("Choose a device first,\nthen reselect your file(s)!")
                            } else {
                                text("")
                            }
                        },
                    }
                ),
                row!(
                    text(self.recieve_file_status.clone())
                )
            )
        ));

        let tx_rx_status_row = Row::with_children(taildrop_status_elements);

        let content_list = list_column().padding(5).spacing(0)
        .add(Element::from(status_row))
        .add(Element::from(enable_row))
        .add(settings::item(
            "Connected",
            toggler(None, self.connect, |value| {
                Message::ConnectDisconnect(value)
            }),
        ))
        .add(Element::from(taildrop_row))
        .add(Element::from(tx_rx_status_row));

        self.core.applet.popup_container(content_list).into()
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}
