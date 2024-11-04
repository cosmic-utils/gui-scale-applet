mod config;
mod logic;
mod window;

use crate::window::Window;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Window>(())?;

    Ok(())
}
