use std::cell::LazyCell;
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

fn get_proxy<'a>() -> LazyCell<Result<ScreenSaverProxyBlocking<'a>>> {
    LazyCell::new(|| {
        let connection = Connection::session()?;
        let proxy = ScreenSaverProxyBlocking::new(&connection)?;
        Ok(proxy)
    })
}

/// Keep screen awake by inhibiting `org.freedesktop.ScreenSaver`
pub struct Caffeine {
    cookie: Option<u32>,
}

impl Caffeine {
    pub fn new() -> Self {
        Self { cookie: None }
    }

    pub fn caffeinate(&mut self) -> Result<()> {
        let proxy = get_proxy();
        let proxy = match proxy.as_ref() {
            Ok(proxy) => proxy,
            Err(e) => return Err(e.clone()),
        };
        let cookie = proxy.inhibit(
            env!("CARGO_PKG_NAME"),
            concat!("Inhibited via ", env!("CARGO_PKG_NAME")),
        )?;
        self.cookie = Some(cookie);
        Ok(())
    }

    pub fn decaffeinate(&mut self) -> Result<()> {
        if let Some(cookie) = self.cookie {
            let proxy = get_proxy();
            let proxy = match proxy.as_ref() {
                Ok(proxy) => proxy,
                Err(e) => return Err(e.clone()),
            };
            proxy.un_inhibit(cookie)?;
            self.cookie = None;
        }
        Ok(())
    }

    pub fn is_caffeinated(&self) -> bool {
        self.cookie.is_some()
    }
}

impl Drop for Caffeine {
    fn drop(&mut self) {
        let _ = self.decaffeinate().unwrap();
    }
}

impl Default for Caffeine {
    fn default() -> Self {
        Self::new()
    }
}
