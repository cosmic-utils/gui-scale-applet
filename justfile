name := 'gui-scale-applet'
<<<<<<< HEAD
appid := 'com.bhh32.gui-scale-applet'
manifest := appid + '.yml'
local-manifest := appid + '.local.yml'
build-dir := 'flatpak-build'
repo := 'repo'

# Default recipe which runs 'just build'
default: build

# Build the flatpak
build: install-deps    
    flatpak-builder --force-clean {{ build-dir }} {{ manifest }}

# Build the flatpak from local source
build-local: install-deps
    flatpak-builder --force-clean {{ build-dir }} {{ local-manifest }}

# Build and install the flatpak for the current user
install: build
    flatpak-builder --user --install --force-clean {{ build-dir }} {{ manifest }}

# Build and install the flatpak from local source for the current user
install-local:
    flatpak-builder --user --install --force-clean {{ build-dir }} {{ local-manifest }}

# Uninstall the flatpak
uninstall:
    flatpak uninstall --user {{ appid }}

# Run the installed flatpak
run:
    flatpak run {{ appid }}
=======
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
>>>>>>> 2a299e6 (Update justfile, deps, and max-popup size)

# Build and export to a local repository
export: build
    flatpak-builder --repo={{ repo }} --force-clean {{ build-dir }} {{ manifest }}

# Create a single-file bundle from the repository
bundle: export
    flatpak build-bundle {{ repo }} {{ name }}.flatpak {{ appid }}

# Clean build artifacts
clean:
    rm -rf {{ build-dir }} {{ repo }} .flatpak-builder {{ name }}.flatpak

# Install required runtime dependencies for the flatpak bundle
install-deps: 
    #!/usr/bin/env bash
    flatpak install --user -y flathub org.freedesktop.Platform//24.08
    flatpak install --user -y flathub org.freedesktop.Sdk//24.08
    flatpak install --user -y flathub org.freedesktop.Sdk.Extension.rust-stable/x86_64/24.08

    xkb="false"
    fb="false"
    # Check for libxkbcommon-dev installed or not
    if command -v pkg-config > /dev/null 2>&1 && pkg-config --exists xkbcommon; then
        echo "libxkbcommon-dev already installed."
        xkb="true"
    else
        echo "libxkbcommon-dev needs installed. Trying automatic installation."
    fi

<<<<<<< HEAD
    if command -v pkg-config > /dev/null 2>&1 && pkg-config --exists flatpak-builder; then
        echo "flatpak-builder already installed."
        fb="true"
    else
        echo "flatpak-builder needs installed. Trying automatic installation."
    fi
    
    if [[ "$xkb" == "true" ]]; then
        if [[ "$fb" == "true" ]]; then
            echo "All system dependencies installed."
            exit 0
        fi
    fi
    
    # Detect distribution ID
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "Attempting to install system dependencies for distribution: $ID"
=======
# Compiles with debug profile
build-debug *args:
    cargo build {{ args }}
>>>>>>> 2a299e6 (Update justfile, deps, and max-popup size)

        case "$ID" in
            pop)
                sudo apt-get update && sudo apt-get install -y libxkbcommon-dev flatpak-builder
                ;;
            fedora)
                sudo dnf install -y libxkbcommon-devel flatpak-builder
                ;;
            arch|manjaro|endeavouros|garuda)
                sudo pacman -S --needed libxkbcommon flatpak-builder
                ;;
            opensuse*|suse)
                sudo zypper install -y libxkbcommon-devel flatpak-builder
                ;;
            *)
                echo "Distribution '$ID' detected but not automatically supported."
                echo "Please install missing system dependencies listed above and try again."
                exit 1
                ;;
            esac            
    else
        echo "Cannot detect distribution."
        echo "Please install all system dependencies listed above manually and try again."
        exit 1
    fi

# Install the flatpak bundle (installs deps first)
install-bundle: install-deps
    flatpak install --user -y {{ name }}.flatpak

# Runs clippy check (for development)
check *args:
    cargo clippy --all-features {{ args }} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Format and run locally with cargo (for development)
dev *args:
    cargo fmt
<<<<<<< HEAD
    env RUST_LOG=cosmic_tasks=info RUST_BACKTRACE=full cargo run --release {{ args }}
=======
    just run {{ args }}

# Run with debug logs
run *args:
    env RUST_LOG=cosmic_tasks=info RUST_BACKTRACE=full cargo run --release {{ args }}

# Installs files
install:
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
>>>>>>> 2a299e6 (Update justfile, deps, and max-popup size)
