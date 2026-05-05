A secure, memory-safe implementation of a Tailscale GUI managmement applet.  

## Links
[GitHub Repo](https://github.com/cosmic-utils/gui-scale-applet)  
[Releases](https://github.com/cosmic-utils/gui-scale-applet/releases)  

## Screenshots
![connections tab]({{ site.baseurl }}/screenshots/connections_tab.png)  
![taildrop tab]({{ site.baseurl }}/screenshots/taildrop_tab.png)  
![devices tab]({{ site.baseurl }}/screenshots/devices_tab.png)  
![settings_tab]({{ site.baseurl }}/screenshots/settings_tab.png)  

## Installation
1. Download the latest flatpak bundle.
2. Install the required runtime dependency:  

```bash
flatpak install --user flathub org.freedesktop.Platform//24.08
```

3. Install the flatpak bundle:  

```bash
flatpak install --user gui-scale-applet.flatpak
```

## Requirements
The applet purposesly avoids running Tailscale commands as root. For it to work you
must first run the following command using the CLI:  

```bash
sudo tailscale set --operator=$USER
```

## Build from Source
1. Clone the repository:
```bash
git clone https://github.com/cosmic-utils/gui-scale-applet.git
cd gui-scale-applet
```
2. Build and install:  
```bash
just install-local
```
Or create a distributable bundle:  
```bash
just bundle
```

## Uninstall
```bash
flatpak uninstall --user com.bhh32.gui-scale-applet
```
