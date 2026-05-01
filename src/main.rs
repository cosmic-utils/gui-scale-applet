use gui_scale_applet::{localize, window::Window};

fn main() -> cosmic::iced::Result {
    localize::localize();
    cosmic::applet::run::<Window>(())?;

    Ok(())
}
