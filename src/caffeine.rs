use cosmic_time::once_cell::sync::Lazy;
use zbus::{blocking::Connection, proxy, Result};

#[proxy(
    interface = "org.freedesktop.ScreenSaver",
    default_service = "org.freedesktop.ScreenSaver",
    default_path = "/ScreenSaver"
)]
trait ScreenSaver {
    /// Inhibit the screensaver
    fn inhibit(&self, application_name: &str, reason: &str) -> Result<u32>;

    /// Uninhibit the screensaver using the cookie from a previous inhibit call
    fn un_inhibit(&self, cookie: u32) -> Result<()>;
}

static PROXY: Lazy<Result<ScreenSaverProxyBlocking<'_>>> = Lazy::new(|| {
    let conn = Connection::session()?;
    Ok(ScreenSaverProxyBlocking::new(&conn)?)
});

fn get_proxy() -> Result<&'static ScreenSaverProxyBlocking<'static>> {
    let proxy = match PROXY.as_ref() {
        Ok(proxy) => proxy,
        Err(e) => return Err(e.clone()),
    };
    Ok(proxy)
}

#[derive(Clone, Default)]
/// Keep screen awake by inhibiting `org.freedesktop.ScreenSaver`
pub struct Caffeine {
    cookie: Option<u32>,
}

impl Caffeine {
    pub fn caffeinate(&mut self) -> Result<()> {
        let proxy = get_proxy()?;
        let cookie = proxy.inhibit(
            env!("CARGO_PKG_NAME"),
            concat!("Inhibited via ", env!("CARGO_PKG_NAME")),
        )?;
        self.cookie = Some(cookie);
        Ok(())
    }

    pub fn decaffeinate(&mut self) -> Result<()> {
        if let Some(cookie) = self.cookie {
            let proxy = get_proxy()?;
            proxy.un_inhibit(cookie)?;
            self.cookie = None;
        }
        Ok(())
    }

    pub fn is_caffeinated(&self) -> bool {
        self.cookie.is_some()
    }

    pub fn cleanup(&self) -> Result<()> {
        if let Some(cookie) = self.cookie {
            let proxy = get_proxy()?;
            proxy.un_inhibit(cookie)?;
        }
        Ok(())
    }
}
