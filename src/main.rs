mod config;
mod logic;
mod window;
pub(crate) mod de;
mod tray;

use crate::tray::setup_tray_icon;
use crate::window::Window;

fn main() -> cosmic::iced::Result {
    // Setup the tray/panel icon for the DEs.
    setup_tray_icon();
    // Run the applet
    cosmic::applet::run::<Window>(())?;

    Ok(())
}
