# GUI Scale About
This is a COSMIC applet for Tailscale. It has SSH and Allow Routes enable/disable and Tail Drop functionality.

## Dependencies
You must first have Tailscale installed and then run:

```bash
sudo tailscale set --operator=$USER
```

This makes it where the applet doesn't need sudo to do it's job.

## Screenshots

![gui-scale-applet-panel](/screenshots/gui-scale-1.0.0-panel.png)  
![gui-scale-applet-open](/screenshots/gui-scale-1.0.0-open.png)  

## How to Install
### Fedora/Fedora based distros
There are two ways to install, you can download the rpm from the releases section or add the Copr repo ***recommended since it stays up-to-date with new features as they are released***:

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