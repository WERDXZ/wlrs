# WLRS Installation and Usage Guide

This guide provides detailed installation instructions and usage examples for WLRS, a dynamic wallpaper engine for Wayland compositors.

## System Requirements

- Linux with a Wayland compositor (e.g., Sway, Hyprland, GNOME Wayland, KDE Wayland)
- Rust compiler 1.70 or newer (for building from source)
- Graphics hardware with Vulkan or OpenGL support
- Wayland development libraries

## Installation

### Installing Dependencies

#### Debian/Ubuntu

```bash
sudo apt install build-essential libwayland-dev libvulkan-dev
```

#### Fedora

```bash
sudo dnf install @development-tools wayland-devel vulkan-loader-devel
```

#### Arch Linux

```bash
sudo pacman -S base-devel wayland vulkan-icd-loader vulkan-headers
```

### Building from Source

1. Clone the repository:

```bash
git clone https://github.com/yourusername/wlrs.git
cd wlrs
```

2. Build the project:

```bash
cargo build --release
```

3. Install the binaries:

```bash
cargo install --path .
```

This will install both `wlrs` (the client) and `wlrs-daemon` (the background service) to your system.

### Installing Example Wallpapers

The repository includes some example wallpapers. Copy them to the WLRS wallpapers directory:

```bash
mkdir -p ~/.local/share/wlrs/wallpapers
cp -r examples/wallpapers/* ~/.local/share/wlrs/wallpapers/
```

## Usage

### Starting the Daemon

First, start the WLRS daemon service:

```bash
wlrs-daemon
```

For debugging or more verbose output:

```bash
RUST_LOG=debug wlrs-daemon
```

It's recommended to add this to your compositor's autostart configuration.

### Basic Commands

#### Listing Available Wallpapers

```bash
wlrs list-wallpapers
```

#### Setting a Wallpaper

```bash
wlrs set-wallpaper "Wallpaper Name"
```

Replace "Wallpaper Name" with the name of one of your available wallpapers.

#### Setting a Wallpaper for a Specific Monitor

```bash
wlrs set-wallpaper "Wallpaper Name" --monitor "Monitor Name"
```

To find your monitor names:

```bash
wlrs query
```

#### Getting Information about Active Wallpapers

```bash
wlrs query
```

This will show all active wallpapers and the monitors they're on.

#### Stopping the Daemon

```bash
wlrs stop-server
```

### Advanced Usage

#### Loading a Wallpaper from a Custom Location

```bash
wlrs load-wallpaper --path /path/to/wallpaper
```

#### Getting Installation Directory

To find where WLRS looks for wallpapers:

```bash
wlrs get-install-directory
```

## Autostarting WLRS

### Sway

Add to your Sway config (`~/.config/sway/config`):

```
exec wlrs-daemon
```

### Hyprland

Add to your Hyprland config (`~/.config/hypr/hyprland.conf`):

```
exec-once = wlrs-daemon
```

### GNOME with systemd

Create a user service file:

```bash
mkdir -p ~/.config/systemd/user/
cat > ~/.config/systemd/user/wlrs.service << EOF
[Unit]
Description=WLRS Wallpaper Engine
PartOf=graphical-session.target

[Service]
ExecStart=/path/to/wlrs-daemon
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF

systemctl --user enable wlrs.service
systemctl --user start wlrs.service
```

## Troubleshooting

### Common Issues

#### "No compositor available" Error

This usually means WLRS can't connect to the Wayland compositor. Ensure you're running in a Wayland session and not X11.

#### Wallpaper Not Showing

Check if your compositor supports the layer-shell protocol. Most modern Wayland compositors do.

#### Wallpaper Not Found

Make sure the wallpaper directory exists and contains a valid manifest.toml file.

#### High CPU/GPU Usage

For animated wallpapers, consider:
- Lowering the FPS in the manifest file
- Simplifying the effects
- Using a static wallpaper instead

### Viewing Logs

For more detailed logs:

```bash
RUST_LOG=debug wlrs-daemon
```

## Command Reference

| Command | Description |
|---------|-------------|
| `wlrs list-wallpapers` | List all available wallpapers |
| `wlrs set-wallpaper "Name"` | Set wallpaper on all monitors |
| `wlrs set-wallpaper "Name" --monitor "Monitor"` | Set wallpaper on specific monitor |
| `wlrs query` | Show active wallpapers and monitors |
| `wlrs load-wallpaper --path /path/to/wallpaper` | Load wallpaper from custom path |
| `wlrs get-install-directory` | Show installation directory |
| `wlrs stop-server` | Stop the daemon |

## Further Resources

- [Creating Custom Wallpapers](creating_wallpapers.md) - Guide to creating your own wallpapers
- [Implementation Notes](../IMPLEMENTATION_NOTES.md) - Technical details for developers
- [README](../README.md) - Project overview and features