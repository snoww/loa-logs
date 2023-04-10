# <img src="https://cdn.discordapp.com/attachments/537415745198489633/1094617120538644622/icon.png" width="30"/> LOA Logs

LOA Logs is a "blazingly fast" open source Lost Ark DPS meter, written in Rust by [Snow](https://github.com/snoww). 

This project is an opinionated flavor of [LOA Details](https://github.com/lost-ark-dev/loa-details) by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing is still done by LOA Details' [`meter-core`](https://github.com/lost-ark-dev/meter-core) under the hood, but the data processing is done using Rust. There are future plans to port the packet sniffing part to Rust as well.

A top priority of this project was to make sure the hell raiding expereince is optimized.

## Download
[https://github.com/snoww/loa-logs/releases](https://github.com/snoww/loa-logs/releases)

*currently only Windows 7 and up is supported

## Prerequisites
LOA Logs require the same prerequisites as LOA Details. You must install Npcap.

Follow instructions [here](https://github.com/lost-ark-dev/loa-details#requirements).

## Screenshots
### In-game Overlay
![log_image](https://cdn.discordapp.com/attachments/537415745198489633/1094551714629173268/image.png)

### Past Encounters
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1094550514152263720/LOA_Logs_Cfivu6fpBy.png" width="500"/>

### Damage Breakdown
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1094550514466832464/LOA_Logs_8GoaTFKkDu.png" width="500"/>

### Skill Cast Log
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1095046175171813436/image.png" width="500"/>

### DPS Log
<img src="https://cdn.discordapp.com/attachments/537415745198489633/1095048314614984785/image.png" width="500"/>
