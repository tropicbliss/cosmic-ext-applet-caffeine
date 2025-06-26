name := 'cosmic-ext-applet-caffeine'
export APPID := 'net.tropicbliss.CosmicExtAppletCaffeine'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

export INSTALL_DIR := base-dir / 'share'

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := APPID + '.desktop'
desktop-src := 'res' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

icon-empty := APPID + '-empty.svg'
icon-empty-src := 'res' / icon-empty
icon-empty-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / icon-empty

icon-full := APPID + '-full.svg'
icon-full-src := 'res' / icon-full
icon-full-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / icon-full

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Installs files
install:
    rm -rf ~/.config/net.tropicbliss.cosmicextappletcaffeine
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{icon-empty-src}} {{icon-empty-dst}}
    install -Dm0644 {{icon-full-src}} {{icon-full-dst}}

# Uninstalls installed files
uninstall:
    rm {{bin-dst}}
    rm {{desktop-dst}}
    rm {{icon-empty-dst}}
    rm {{icon-full-dst}}
