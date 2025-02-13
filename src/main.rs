mod config;
mod logic;
mod window;
pub(crate) mod de;

use crate::window::Window;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Window>(())?;

    Ok(())
}
