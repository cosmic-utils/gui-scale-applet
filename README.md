# GUI Scale COSMIC Desktop Applet

## About

A secure, memory-safe implementation of a Tailscale GUI management applet for the System76 COSMIC desktop environment. This project demonstrates secure systems programming practices while providing essential functionality for Tailscale network (tailnet) management.

### Project Overview

The GUI Scale applet provides a user-friendly interface for managing Tailscale configurations within the COSMIC desktop environment. It showcases secure systems programming practices while maintaining high usability standards.

### Key Features

- Secure Tailscale network (tailnet) managment
- Memory-safe implementation in Rust
    - Utilizes Rust's ownership system for memory-safety operations
    - No unsafe code blocks used
    - All external data properly validated
    - Buffer overflow protection through Rust's bounds checking
- Comprehensive error handling
- Integration with system security policies
    - Proper permission handling for system operations
        - Does not run while the Tailscale operator configuration is set to root.
    - Minimal system privilege requirements
        - Does not run as root

### Security Architecture

The applet implements a layered security approach:

1. Privilege Management
    - Minimal required permissions
    - Proper capabilitiy isolation
2. Error Handling
    - Uses Rust's built-in error types (Result and Option), as well as return other types, such as String and bool, to test for and handle any errors that occurred during runtime
    - No information leakage

## Dependencies

You must first have Tailscale installed and then run:

```bash
sudo tailscale set --operator=$USER
```

This makes it where the applet doesn't need sudo (root) to do its job.

## Screenshots

![gui-scale-applet-panel](/screenshots/gui-scale-panel.png)
![gui-scale-applet-open](/screenshots/gui-scale-applet-open.png)

## Installation

### Fedora/Fedora based distros

Add the Copr repo:

```bash
sudo dnf copr enable bhh32/gui-scale-applet
sudo dnf update --refresh
sudo dnf install -y gui-scale-applet
```

### Debian/Ubuntu (including Pop!OS) based Distros

Unfortunately, I don't know anything like Copr for these distros, so you can download the deb package from the releases section of this repo.

### Other

For any other distros (except atomic/immutable distros) you can run:

```bash
git clone https://github.com/cosmic-utils/gui-scale-applet.git
cd gui-scale-applet
sudo just install
```
