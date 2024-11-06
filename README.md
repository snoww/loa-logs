# <img src="https://i.imgur.com/q2O4nnn.png" width="30"/> LOA Logs

[![GitHub](https://img.shields.io/github/downloads/snoww/loa-logs/total?style=for-the-badge&color=%23ff9800)](https://github.com/snoww/loa-logs/releases/latest) [![Discord](https://img.shields.io/discord/1174544914139328572?color=%235865F2&label=Discord&style=for-the-badge)](https://discord.gg/RXvTMV2YHu)

[![GitHub](https://img.shields.io/github/v/release/snoww/loa-logs?style=flat-square)](https://github.com/snoww/loa-logs/releases)
[![GitHub](https://img.shields.io/github/license/snoww/loa-logs?style=flat-square)](https://github.com/snoww/loa-logs/blob/master/LICENSE)

[<img src="static/kofi.png" alt="Ko-fi" width="230"/>](https://ko-fi.com/synow)

[!["Buy Me A Coffee"](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/synow)

LOA Logs is a "blazingly fast" open source Lost Ark DPS meter, written in Rust by [Snow](https://github.com/snoww).

This project is an opinionated flavor of [LOA Details](https://github.com/lost-ark-dev/loa-details) by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing and processing has been completely ported over to Rust, with [`meter-core-rs`](https://github.com/snoww/meter-core-rs). The Rust port could not be made without Herysia and Henjuro's work on [`meter-core`](https://github.com/lost-ark-dev/meter-core).

This project was designed specifically with hell-raiding in mind.

# Download

https://github.com/snoww/loa-logs/releases

\*currently only Windows 7 and up is supported

> [!IMPORTANT]
>
> ### Prerequisites
>
> Npcap is required to run LOA Logs.
>
> Download [here](https://npcap.com/#download).

# Supporting the Project

You can support me directly by buying me a [coffee.](https://www.buymeacoffee.com/synow)

# FAQ

#### Q: METER NOT WORKING AFTER MAJOR PATCH!!!

A: This is normal. The meter will not work after a major game patch. The game shuffles around the opcodes and packets every major patch, and the meter must be updated in order for it to work. Please wait patiently until the meter update is ready. If you keep meter open a pop-up should show up prompting you to update once it is released.

#### Q: Missing `WinDivert64.sys`

A: You need to reinstall meter. The meter uses the WinDivert driver to listen to game packets. You either removed the file or your antivirus removed it. 

#### Q: Meter isn't detecting anything...

A: There can be multiple reasons. If you have NordVPN installed, meter will not work due to both apps using WinDivert. You need to uninstall Nord, or completely quit the Nord processes and reboot.

#### Q: How to use ExitLag with LOA Logs?

A: ExitLag recently updated their settings which changed how they redirect packets. Change your ExitLag settings to _Packet redirection method > Legacy - NDIS_. 

#### Q: How to use other ping reducers with LOA Logs?

A: If there is an option to use NDIS packet redirection in your app, select that setting. 

#### Q: How to use a traditional VPN with LOA Logs?

A: Traditional VPNs (NordVPN, Private Internet Access, etc.) are no longer supported anymore due to changes in packet requirements. They cannot be running at the same time as the meter.

#### Q: Class not swapping or character name is stuck on the previous character

A: Are you using raw socket? Raw socket is wonky and has frequent packet losses, and this occurs often during character swaps. If you can run the meter without raw socket then please turn it off. If your meter doesn't work without raw socket, then unfortunately the issue will persist.

#### Q: Should I run it in a VM?

A: Can no longer run meter in a VM due to changes in packet requirements.

#### Q: Meter window is missing / meter window is tiny

A: Right-click the taskbar icon (located in the bottom right of your screen, next to the system time), click reset position, or load saved position. Adjust the size of the window and location, and then save the position.

#### Q: The installer crashes or takes forever to install

A: Are you trying to install on a custom install folder with different permissions? You might need to run the installer in administrator mode due to permission issues.

#### Q: The meter crashes immediately when trying to open it. EdgeWebview2 Error.

A: There could be two possible reasons. 1. The meter needs Microsoft Edge Webview2 Runtime to run. Yours is probably missing or out of date. Go uninstall it first (it won't let you install it if you have an older version installed), then download and install from [here](https://go.microsoft.com/fwlink/p/?LinkId=2124703) (https://go.microsoft.com/fwlink/p/?LinkId=2124703). 2. If you installed the meter in another folder that might require elevated permissions, you would need to run the program in administrator mode.

#### Q: The meter window lags a lot when dragging around.

A: Are you on Windows 11? Disable blur in the settings (settings > accessibility). If you wish to have a dark background with blur disabled, also disable the transparency setting to have a pseudo dark mode.

#### Q: Why isn't my item level shown next to my name when others have it?

A: You opened the meter too late, and it wasn't able to get your character information. It is doing its best by guessing. You can fix this by: switching characters, or changing parties around. (note: you need to enable "show gear score" in settings to show item level)

#### Q: There are too many/too few columns in the meter.

A: You can change whatever column you want to show in the settings. TIP: you can `SHIFT+SCROLL` to scroll horizontally.

#### Q: Are you going to implement rDPS like LOA Details?

A: rDPS is no longer working due to missing packets.

#### Q: Help, my issue isn't listed here.

A: Create an issue here on GitHub, or send a message in the #troubleshooting channel on Discord. [(invite)](https://discord.gg/HMtnzPFHTG)

#### Q: Is it really "blazingly fast"?

A: [Yes.](https://i.imgur.com/QsLAntt.png)

## Screenshots

### In-game Overlay (optional Boss HP bar)

![log_image](https://i.imgur.com/luHu7Fz.png)

### Damage Breakdown with DPS Charts

<img src="https://i.imgur.com/T4HX6XK.png" width="500"/>

### rDPS

<img src="https://i.imgur.com/cxKz9pP.png"/>

### Skill Breakdown

<img src="https://i.imgur.com/P5Mb9oe.png" width="600"/>

### Arcana Card Tracking

<img src="https://i.imgur.com/afoAVOZ.png" width="500"/>

### Buff Uptime Tracking

<img src="https://i.imgur.com/9SkFQs3.png" width="800"/>

### Opener Rotation

<img src="https://i.imgur.com/hcpHAKG.png" width="600"/>

### Past Encounters

<img src="https://i.imgur.com/RZT6Rww.png" width="500"/>

#### Search Filters

<img src="https://i.imgur.com/5aJJISG.png" width="400"/>
