# Caffeine Applet for the COSMIC™ desktop

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

## Additional Credits
- App indicator icon is taken from the [Caffeine GNOME extension](https://github.com/eonpatapon/gnome-shell-extension-caffeine) project and slightly modified
