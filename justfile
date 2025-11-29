name := 'gui-scale-applet'
export APPID := 'com.github.bhh32.GUIScaleApplet'
rootdir := ''
prefix := '/usr'
base-dir := absolute_path(clean(rootdir / prefix))
export INSTALL_DIR := base-dir / 'share'
bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name
desktop := APPID + '.desktop'
desktop-src := 'data' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop
metainfo := APPID + '.metainfo.xml'
metainfo-src := 'data' / metainfo
metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo
icon := 'tailscale-icon.png'
icons-src := 'data' / 'icons' / 'scalable' / 'apps' / icon
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor' / 'scalable' / 'status' / icon

# Default recipe which runs 'just build-release'
default: build-release

# Runs 'cargo clean'
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# 'cargo clean' and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build *args:
    cargo build {{ args }}

# Compiles with release profile
build-release *args: (build '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs clippy check
check *args:
    cargo clippy --all-features {{ args }} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

dev *args:
    cargo fmt
    just run {{ args }}

# Run with debug logs
run *args:
    env RUST_LOG=cosmic_tasks=info RUST_BACKTRACE=full cargo run --release {{ args }}

# Installs files
install: (build-release)
    sudo install -Dm0755 {{ bin-src }} {{ bin-dst }}
    sudo install -Dm0644 {{ desktop-src }} {{ desktop-dst }}
    sudo install -Dm0644 {{ icons-src }} {{ icons-dst }}

# Uninstalls installed files
uninstall:
    sudo rm {{ bin-dst }}
    sudo rm {{ desktop-dst }}
    sudo rm {{ icons-dst }}

# Vendor dependencies only
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extract vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
