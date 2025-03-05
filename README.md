# <img src="https://i.imgur.com/VzbJVpr.png" width="30"/> LOA Logs

[![GitHub](https://img.shields.io/github/downloads/snoww/loa-logs/total?style=for-the-badge&color=%23ff9800)](https://github.com/snoww/loa-logs/releases/latest) [![Discord](https://img.shields.io/discord/1174544914139328572?color=%235865F2&label=Discord&style=for-the-badge)](https://discord.gg/RXvTMV2YHu)

[![GitHub](https://img.shields.io/github/v/release/snoww/loa-logs?style=flat-square)](https://github.com/snoww/loa-logs/releases)
[![GitHub](https://img.shields.io/github/license/snoww/loa-logs?style=flat-square)](https://github.com/snoww/loa-logs/blob/master/LICENSE)

[<img src="static/kofi.png" alt="Ko-fi" width="230"/>](https://ko-fi.com/synow)

[!["Buy Me A Coffee"](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/synow)

LOA Logs is a "blazingly fast" open source Lost Ark DPS meter, written in Rust by [Snow](https://github.com/snoww), and many other great contributors from the community.

This project originated as an opinionated flavor of [LOA Details](https://github.com/lost-ark-dev/loa-details) by Herysia and Mathi. However, since then it has been completely independent and rewritten from scratch. The meter couldn't exist without the help from [@poont](https://github.com/Irrialite), [@molenzwiebel](https://github.com/molenzwiebel), [@mathi](https://github.com/Mathicha), and others.

# Download

https://github.com/snoww/loa-logs/releases

\*currently only Windows 7 and up is supported

# Supporting the Project

You can support me directly by buying me a [coffee.](https://www.buymeacoffee.com/synow)

# Contributing to Meter

Due to the nature of the encryption of the packets, the source for meter-core-rs is not public. That means you won't have access to live meter when building on your own. However, you can contribute to the UI and other parts of the project. The frontend is written in [Svelte](https://svelte.dev/), interfacing with the backend through [tauri v1](https://v1.tauri.app/v1/guides/getting-started/prerequisites) in Rust. If you're interested in contributing, please familiarize yourself with the tools and join the discord server using the panel above.

### Prerequisites
- Some version of [Node.js](https://nodejs.org/en/download/)
- tauri & Rust (see [tauri v1 getting started](https://v1.tauri.app/v1/guides/getting-started/prerequisites))
- Clone the repository
- Install dependencies

```bash
npm install
```

### Setup

After everything has been installed, you should be able to build the dev version of the meter. Note, without meter-core-rs, you will not get live meter to show data, however, you can still interact with previously saved logs.

```bash
npm run tauri dev
```

# FAQ

### Table of Contents
- [METER NOT WORKING AFTER MAJOR PATCH!!!](#q-meter-not-working-after-major-patch)
- [Missing `WinDivert64.sys`](#q-missing-windivert64sys)
- [Meter isn't detecting anything...](#q-meter-isnt-detecting-anything)
- [How to use ExitLag?](#q-how-to-use-exitlag-with-loa-logs)
- [How to use other ping reducers?](#q-how-to-use-other-ping-reducers-with-loa-logs)
- [How to use a traditional VPN (e.g. NordVPN)?](#q-how-to-use-a-traditional-vpn-with-loa-logs)
- [Class not swapping or character name is stuck on the previous character](#q-class-not-swapping-or-character-name-is-stuck-on-the-previous-character)
- [Should I run it in a VM?](#q-should-i-run-it-in-a-vm)
- [Meter window is missing / meter window is tiny](#q-meter-window-is-missing--meter-window-is-tiny)
- [The installer crashes or takes forever to install](#q-the-installer-crashes-or-takes-forever-to-install)
- [The meter crashes immediately when trying to open it. EdgeWebview2 Error.](#q-the-meter-crashes-immediately-when-trying-to-open-it-edgewebview2-error)
- [The meter window lags a lot when dragging around.](#q-the-meter-window-lags-a-lot-when-dragging-around)
- [Why isn't my item level shown next to my name when others have it?](#q-why-isnt-my-item-level-shown-next-to-my-name-when-others-have-it)
- [There are too many/too few columns in the meter.](#q-there-are-too-manytoo-few-columns-in-the-meter)
- [rDPS?](#q-are-you-going-to-implement-rdps-like-loa-details)
- [Help, my issue isn't listed here](#q-help-my-issue-isnt-listed-here-or-youve-tried-all-these-solutions-and-it-still-doesnt-work)

#### Q: METER NOT WORKING AFTER MAJOR PATCH!!!

A: This is normal. The meter will not work after a major game patch. The game shuffles around the opcodes and packets every major patch, and the meter must be updated in order for it to work. Please wait patiently until the meter update is ready. If you keep meter open a pop-up should show up prompting you to update once it is released.

#### Q: Missing `WinDivert64.sys`

A: You need to reinstall meter. The meter uses the WinDivert driver to listen to game packets. You either removed the file or your antivirus removed it. Please create an exception for the entire meter folder, and then reinstall the meter. After reinstalling, you should restart your computer before launching meter.

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

#### Q: Help, my issue isn't listed here. Or you've tried all these solutions, and it still doesn't work.

A: Search the message history in the [#troubleshooting]((https://discord.gg/HMtnzPFHTG)) channel on Discord. If you can't find a solution there, please provide your log file and describe your issue. Open Meter > Settings > Database Tab > Open Folder > Copy the `loa_logs_rCURRENT.log` file. The log file does not contain any personal ips other than your local ip, and the ip addresses of the game servers hosted by amazon.

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
