use crate::de::*;
use futures::executor;
use gtk::prelude::*; // Add this line to include gtk module
use libappindicator::{AppIndicator, AppIndicatorStatus};
use zbus::Connection;

pub(crate) fn setup_tray_icon() {
    match detect_desktop_environment() {
        DesktopEnvironment::Gnome | DesktopEnvironment::Xfce => {
            println!("Detected GNOME/XFCE. Using libappindicator.");
            setup_libappindicator();
        }
        DesktopEnvironment::KDE => {
            println!("Detected KDE. Using status-notifier-item.");
            setup_status_notifier();
        }
        DesktopEnvironment::Pantheon => {
            println!("Detected Pantheon. Using status-notifier-item.");
            setup_status_notifier();
        }
        DesktopEnvironment::Cinnamon => {
            println!("Detected Cinnamon. Using libappindicator.");
            setup_status_notifier();
        }
        DesktopEnvironment::Cosmic => {
            println!("Detected COSMIC. Using status-notifier-item.");
            setup_status_notifier();
        }
        DesktopEnvironment::Unknown => {
            println!("Unknown desktop environment. Defaulting to libappindicator.");
            setup_libappindicator();
        }
    }
}

fn setup_libappindicator() {
    use libappindicator::{AppIndicator, AppIndicatorStatus};
    let mut indicator = AppIndicator::new("gui-scale-applet", "tailscale-icon");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_menu(&mut create_menu());
}

fn setup_status_notifier() {
    use zbus::Connection;
    if let Ok(connection) = executor::block_on(Connection::session()) {
        if let Err(e) = executor::block_on(connection.call_method(
            Some("org.kde.StatusNotifierWatcher"),
            "/StatusNotifierWatcher",
            Some("org.kde.StatusNotifierWatcher"),
            "RegisterStatusNotifierItem",
            &("com.github.bhh32.GUIScaleApplet"),
        )) {
            eprintln!("Failed to register StatusNotifierItem: {}", e);
        }
    } else {
        eprintln!("Failed to connect to D-Bus session");
    }
}

pub(crate) fn create_menu() -> gtk::Menu {
    let _ = gtk::init().expect("Failed to initialize GTK.");
    let menu = gtk::Menu::new();
    let quit_item = gtk::MenuItem::with_label("Quit");
    quit_item.connect_activate(|_| std::process::exit(0));
    menu.append(&quit_item);
    menu.show_all();
    menu
}
