# <img src="https://cdn.discordapp.com/attachments/537415745198489633/1094617120538644622/icon.png" width="30"/> LOA Logs

[![GitHub](https://img.shields.io/github/downloads/snoww/loa-logs/total?style=for-the-badge)](https://github.com/snoww/loa-logs/releases/latest)


![GitHub](https://img.shields.io/github/v/release/snoww/loa-logs?style=flat-square)
![GitHub](https://img.shields.io/github/license/snoww/loa-logs?style=flat-square)

LOA Logs is a "blazingly fast" open source Lost Ark DPS meter, written in Rust by [Snow](https://github.com/snoww). 

This project is an opinionated flavor of [LOA Details](https://github.com/lost-ark-dev/loa-details) by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing and processing has been completely ported over to Rust, with [`meter-core-rs`](https://github.com/snoww/meter-core-rs). A huge thanks to Herysia and Henjuro for their work on the original [`meter-core`](https://github.com/lost-ark-dev/meter-core). This gives the meter huge performance improvements with low memory usage compared the TypeScript implementation.


This project was designed specifically with hell-raiding in mind.

## Download
[https://github.com/snoww/loa-logs/releases](https://github.com/snoww/loa-logs/releases)

*currently only Windows 7 and up is supported

## Prerequisites
LOA Logs require the same prerequisites as LOA Details. You must install Npcap.

Follow instructions [here](https://github.com/lost-ark-dev/loa-details#requirements).

## Screenshots
### In-game Overlay (optional Boss HP bar)
![log_image](https://cdn.discordapp.com/attachments/537415745198489633/1100293328995614750/image.png)

### Past Encounters
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100229380652929044/image.png" width="500"/>

### Damage Breakdown with DPS Charts
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100220743846989935/image.png" width="500"/>

### Skill Breakdown
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100230099640524861/image.png" width="500"/>

### Buff Uptime Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100220998378324068/image.png" width="500"/>

### Identity Tracking
#### Arcana Card Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100220506231287818/image.png" width="500"/>

#### Bard/Artist Bubble Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100239509754490931/image.png" width="500"/>

### Stagger Tracking
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1100300320392871986/image.png" width="500"/>

### Skill Cast Log
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1095046175171813436/image.png" width="500"/>
