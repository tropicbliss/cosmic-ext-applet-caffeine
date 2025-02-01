use crate::localize::localize;
use crate::window::Window;

mod caffeine;
mod localize;
mod window;

fn main() -> cosmic::iced::Result {
    localize();
    cosmic::applet::run::<Window>(false, ())
}
