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
desktop-dst := base-dir / 'share' / 'applications' / desktop

icon-empty := APPID + '-empty.svg'
icon-empty-src := 'res' / icon-empty
icon-empty-dst := base-dir / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / icon-empty

icon-full := APPID + '-full.svg'
icon-full-src := 'res' / icon-full
icon-full-dst := base-dir / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / icon-full

metainfo := APPID + '.metainfo.xml'
metainfo-src := 'res' / metainfo
metainfo-dst := base-dir / 'share' / 'metainfo' / metainfo

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
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}

# Uninstalls installed files
uninstall:
    rm {{bin-dst}}
    rm {{desktop-dst}}
    rm {{icon-empty-dst}}
    rm {{icon-full-dst}}

vendor-generate:
    python3 ./flatpak/flatpak-cargo-generator.py ./Cargo.lock -o ./flatpak/cargo-sources.json

validate:
    appstreamcli validate --pedantic --explain {{metainfo-src}}

flatpak:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    just build-release --frozen --offline
    just prefix=/app install

flatpak-local:
    flatpak-builder --force-clean build-dir {{APPID}}.yml
    flatpak-builder --user --install --force-clean build-dir {{APPID}}.yml
    flatpak run {{APPID}}
