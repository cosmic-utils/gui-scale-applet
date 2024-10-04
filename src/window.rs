use cosmic::app::Core;
use cosmic::iced::widget;
use cosmic::iced::{
    wayland::popup::{destroy_popup, get_popup},
    window::Id,
    Command, 
    Limits
};
use cosmic::iced_runtime::core::window;
use cosmic::iced_style::application;
use cosmic::widget::{list_column, settings, text, toggler};
use cosmic::{Element, Theme};
use crate::logic::{
    get_tailscale_con_status, 
    get_tailscale_ip, 
    get_tailscale_routes_status, 
    get_tailscale_ssh_status,
    get_tailscale_devices,
    set_ssh,
    set_routes,
    tailscale_int_up,
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
            selected_device: "No Selection".to_string(),
            
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

        let content_list = list_column().padding(5).spacing(0).add(settings::item(
            "IPv4 Address",
            text(ip.clone()),
            ),
        )
        .add(settings::item(
            "Connection Status",
            text(if conn_status { "Tailscale Connected" } else { "Tailscale Disconnected" })
        ))
        .add(settings::item(
            "Enable SSH",
            toggler(None, self.ssh, |value| {
                Message::EnableSSH(value)
            }),
        ))
        .add(settings::item(
            "Accept Routes",
            toggler(None, self.routes, |value| {
                Message::AcceptRoutes(value)
            }),
        ))
        .add(settings::item(
            "Connected",
            toggler(None, self.connect, |value| {
                Message::ConnectDisconnect(value)
            }),
        ))
        .add(settings::item(
            "Devices",
            cosmic::widget::combo_box(&self.device_state, "No Selection", Some(&self.selected_device), Message::DeviceSelected)
                .width(cosmic::iced::Length::Fill)
        ));

        self.core.applet.popup_container(content_list).into()
    }

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}
