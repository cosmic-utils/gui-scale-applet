name := 'gui-scale-applet'
appid := 'com.bhh32.gui-scale-applet'
manifest := appid + '.yml'
local-manifest := appid + '.local.yml'
build-dir := 'flatpak-build'
repo := 'repo'

# Default recipe which runs 'just build'
default: build

# Build the flatpak
build:
    flatpak-builder --force-clean {{ build-dir }} {{ manifest }}

# Build the flatpak from local source
build-local:
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
    flatpak install --user -y flathub org.freedesktop.Platform//24.08

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
    env RUST_LOG=cosmic_tasks=info RUST_BACKTRACE=full cargo run --release {{ args }}
