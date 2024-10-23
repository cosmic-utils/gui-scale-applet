mod window;
mod logic;
mod config;

use crate::window::Window;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Window>(())?;

    Ok(())
}
