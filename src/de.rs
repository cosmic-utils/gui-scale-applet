use std::env;

pub enum DesktopEnvironment {
    Gnome,
    Xfce,
    KDE,
    Pantheon,
    Cinnamon,
    Cosmic,
    Unknown,
}

pub fn detect_desktop_environment() -> DesktopEnvironment {
    if let Ok(current_desktop) = env::var("XDG_CURRENT_DESKTOP") {
        if current_desktop.contains("GNOME") {
            return DesktopEnvironment::Gnome;
        } else if current_desktop.contains("XFCE") {
            return DesktopEnvironment::Xfce;
        } else if current_desktop.contains("KDE") || current_desktop.contains("Plasma") {
            return DesktopEnvironment::KDE;
        } else if current_desktop.contains("Pantheon") {
            return DesktopEnvironment::Pantheon;
        } else if current_desktop.contains("Cinnamon") {
            return DesktopEnvironment::Cinnamon;
        } else if current_desktop.contains("COSMIC") {
            return DesktopEnvironment::Cosmic;
        }
    }
    DesktopEnvironment::Unknown
}
