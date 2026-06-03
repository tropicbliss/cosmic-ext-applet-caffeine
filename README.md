# Caffeine Applet for the COSMIC™ desktop

## Why am I archiving this repo?
I no longer mainline COSMIC desktop and it has become increasingly obvious that a background daemon is required to support a multi-monitor setup, which is in my opinion going above and beyond what a simple tray applet should do. An alternative solution that does not involve inhibiting D-Bus might be required to maintain a stateless widget. I could try and coordinate syncing between each applet instance for each monitor, but it will be prone to bugs and I'm not sure that I would be willing to code that up for a desktop environment I don't even use anymore. Feel free to fork this repo.

A lightweight system tray application that prevents your screen from going to
sleep. Perfect for presentations, watching videos, or any situation where you
need your display to stay active.

![Screenshot of applet](res/screenshot.png)

## How It Works

This applet uses the D-Bus screensaver interface (org.freedesktop.ScreenSaver)
to tell your system not to activate the screensaver or power saving mode. When
activated, it maintains a screen wake lock that keeps your display on until you
either deactivate it or close the application.

## Install

```bash
git clone https://github.com/tropicbliss/cosmic-ext-applet-caffeine
cd cosmic-ext-applet-caffeine
just build-release
sudo just install
```

If you are using a Debian-based distro, you may need to install some other dependencies first
```bash
sudo apt install cargo cmake just libexpat1-dev libfontconfig-dev libfreetype-dev libxkbcommon-dev pkgconf
```

## Enable the applet
Open COSMIC settings and navigate to Desktop -> Panel -> Configure panel applets.
Caffeine will be available to add from the Add Applet button.

## Additional Credits
- App indicator icon is taken from the [Caffeine GNOME extension](https://github.com/eonpatapon/gnome-shell-extension-caffeine) project and slightly modified.
