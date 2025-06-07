use crate::localize::localize;
use crate::window::Window;

mod caffeine;
mod localize;
mod timer;
mod window;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();
    tracing::info!("Starting caffeine applet with version {VERSION}");

    localize();
    cosmic::applet::run::<Window>(false, ())
}
