# Running LOA Logs on Linux

LOA Logs on Linux is split into two programs:

- Nineveh: backend game packet reader
- LOA Logs: main frontend application, delivered as an AppImage

## Option 1: Automated installer (recommended)

The included install script downloads the latest release, sets up a desktop entry, and creates a launcher that
auto-updates both the AppImage and nineveh on each start.

Requirements: `curl`, `python3`, `pkexec`, `pgrep` (standard on most desktop Linux distributions).

```bash
git clone https://github.com/snoww/loa-logs.git
cd loa-logs/scripts/linux
./install.sh
```

This will:

- Download the latest AppImage and nineveh binary to `~/.local/share/loa-logs/`
- Install a desktop entry so LOA Logs appears in your application menu
- Set up a launcher that automatically checks for updates on each start

To uninstall:

```bash
cd loa-logs/scripts/linux
./uninstall.sh
```

Your encounter data and settings are preserved on uninstall.

## Option 2: Manual setup

Download the newest AppImage and `nineveh` from the [latest release](https://github.com/snoww/loa-logs/releases).

Make both files executable:

```bash
chmod +x nineveh
chmod +x LOA.Logs_1.45.1_amd64.appimage
```

Start Nineveh with root permissions and keep it running in the background:

```bash
sudo ./nineveh --stop-after-timeout 0 --proxy-without-ipc
```

Then open the LOA Logs AppImage.

### Manual updates

The main LOA Logs AppImage should update like it does on Windows.

Nineveh needs to be updated manually after game updates or patches. If LOA Logs stops working after a patch, check the
releases page for a newer Nineveh download.

### Example script

This script assumes you renamed the AppImage to `LOA_Logs.appimage`. Change `location` to your LOA Logs folder before
running it.

```bash
#!/bin/bash

location="COPY WHOLE PATH TO LOGS FOLDER"

cd "$location"

sudo pkexec ./nineveh --stop-after-timeout 0 --proxy-without-ipc &
NINEVEH_PID=$!

while ! pgrep -x "nineveh" > /dev/null; do
  sleep 0.5
done

sleep 1

./"LOA_Logs.appimage"

sudo pkexec kill $NINEVEH_PID
```

## Troubleshooting

Check permissions after each time the app or nineveh is redownloaded.

### LOA Logs opens to a white window / EGL BAD PARAMETER

Preload the Wayland client library:

```bash
LD_PRELOAD=/usr/lib/libwayland-client.so ./"LOA_Logs.appimage"
```

If that does not work, or you are using an immutable distro like Bazzite, copy `libwayland-client.so` to your LOA Logs
folder and preload that file instead:

```bash
LD_PRELOAD=/path/to/logs/folder/libwayland-client.so ./"LOA_Logs.appimage"
```

### Data folder

LOA Logs data is stored here:

```text
/home/$USER/.local/share/xyz.snow.loa-logs/
```

### Issue not listed here

Join the [#linux](https://discord.gg/HMtnzPFHTG) channel on Discord and ask the community members, grab the linux role
from #get-roles.