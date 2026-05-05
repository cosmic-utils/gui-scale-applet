A secure, memory-safe implementation of a Tailscale GUI managmement applet.  

## Links
<a href="https://github.com/cosmic-utils/gui-scale-applet class="pill-btn">GitHub Repo</a>
a href="https://github.com/cosmic-utils/gui-scale-applet/releases" class="pill-btn">Releases</a>

## Screenshots
![connections tab]({{ site.baseurl }}/assets/screenshots/connections_tab.png)  
![taildrop tab]({{ site.baseurl }}/assets/screenshots/taildrop_tab.png)  
![devices tab]({{ site.baseurl }}/assets/screenshots/devices_tab.png)  
![settings_tab]({{ site.baseurl }}/assets/screenshots/settings_tab.png)  

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
