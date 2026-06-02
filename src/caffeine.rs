use zbus::{Connection, proxy};

#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/ScreenSaver"
)]
trait ScreenSaver {
    async fn inhibit(&self, application_name: &str, reason: &str) -> zbus::Result<u32>;
    async fn un_inhibit(&self, cookie: u32) -> zbus::Result<()>;
}

pub async fn inhibit() -> zbus::Result<u32> {
    let connection = Connection::session().await?;
    let proxy = ScreenSaverProxy::new(&connection).await?;
    proxy
        .inhibit(
            env!("CARGO_PKG_NAME"),
            concat!("Inhibited via ", env!("CARGO_PKG_NAME")),
        )
        .await
}

pub async fn uninhibit(cookie: u32) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    let proxy = ScreenSaverProxy::new(&connection).await?;
    proxy.un_inhibit(cookie).await
}

#[derive(Clone, Default)]
pub struct Caffeine {
    cookie: Option<u32>,
}

impl Caffeine {
    pub fn is_caffeinated(&self) -> bool {
        self.cookie.is_some()
    }

    pub fn set_cookie(&mut self, cookie: u32) {
        self.cookie = Some(cookie);
    }

    pub fn clear_cookie(&mut self) {
        self.cookie = None;
    }

    pub fn take_cookie(&mut self) -> Option<u32> {
        self.cookie.take()
    }
}
