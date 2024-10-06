use std::path::PathBuf;
use cosmic::dialog::file_chooser::{self, FileFilter};
use tokio::io::AsyncReadExt;
use cosmic::app::Core;
use cosmic::iced::widget::{
    self, 
    column, 
    row, 
    vertical_space, 
    horizontal_space,
};
use cosmic::iced_widget::{Column, Row};
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
    get_tailscale_con_status, get_tailscale_devices, get_tailscale_ip, get_tailscale_routes_status, get_tailscale_ssh_status, set_routes, set_ssh, tailscale_int_up, tailscale_send
};

const ID: &str = "com.github.bhh32.GUIScaleApplet";

pub struct Window {
    core: Core,
    popup: Option<Id>,
    ssh: bool,
    routes: bool,
    connect: bool,
    device_state: cosmic::widget::combo_box::State<String>,
    selected_device: String,
    send_files: Vec<Option<String>>,
    send_file_status: Vec<Option<String>>
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    UpdateStatusRow(bool),
    EnableSSH(bool),
    AcceptRoutes(bool),
    ConnectDisconnect(bool),
    DeviceSelected(String),
    ChooseFiles,
    FileSelected(Url),
    SendFiles,
    FileChoosingCancelled
}

impl cosmic::Application for Window {
    type Executor = cosmic::SingleThreadExecutor;
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
            selected_device: "No Device".to_string(),
            send_files: Vec::<Option<String>>::new(),
            send_file_status: Vec::<Option<String>>::new(),
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
            Message::UpdateStatusRow(updated) => {},
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
                // Debug Only
                println!("selected Device: {}", self.selected_device);
            }
            Message::ChooseFiles => {
                return cosmic::command::future(async move {
                    let file_filter = FileFilter::new("Any").glob("*.*");
                    let dialog = file_chooser::open::Dialog::new()
                        .title("Choose a file or files...")
                        .filter(file_filter);

                    let msg = match dialog.open_file().await {
                        Ok(file_response) => Message::FileSelected(file_response.url().to_owned()),
                        Err(file_chooser::Error::Cancelled) => Message::FileChoosingCancelled,
                        Err(e) => {
                            eprintln!("Choosing a file or files went wrong: {e}");
                            Message::FileChoosingCancelled
                        }
                    };

                    msg
                });
            }
            Message::FileSelected(url) => {
                let path = match url.to_file_path() {
                    Ok(good_path) => good_path,
                    Err(_e) => {
                        PathBuf::new()
                    }
                };

                if path.exists() {
                    self.send_files.push(Some(match path.as_path().to_str() {
                        Some(p) => String::from(p),
                        None => String::new()
                    }));
                }
            }
            Message::SendFiles => {
                let send_statuses = tailscale_send(self.send_files.clone(), &self.selected_device);

                for status in send_statuses.iter() {
                    self.send_file_status.push(status.clone());
                }
            }
            Message::FileChoosingCancelled => {

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

        let mut taildrop_elements: Vec<Element<'_, Message>> = Vec::new();
        taildrop_elements.push(Element::from(
            column!(
                row!(text("Tail Drop")),
                row!(
                    combo_box(&self.device_state, "No Device", Some(&self.selected_device), Message::DeviceSelected),
                    button::standard("File(s)")
                        .on_press(Message::ChooseFiles)
                ),
                row!(
                    if self.send_files.len() > 0 {
                    button::standard("Send File(s)")
                        .on_press(Message::SendFiles)
                    } else {
                        button::standard("Send File(s)")
                    }
                )
            )
            .spacing(5)
        ));
        
        let taildrop_row = Row::with_children(taildrop_elements);

        let content_list = list_column().padding(5).spacing(0)
        .add(Element::from(status_row))
        .add(Element::from(enable_row))
        .add(settings::item(
            "Connected",
            toggler(None, self.connect, |value| {
                Message::ConnectDisconnect(value)
            }),
        ))
        .add(Element::from(taildrop_row));

        self.core.applet.popup_container(content_list).into()
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}
