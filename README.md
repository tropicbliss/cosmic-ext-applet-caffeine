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

