# LOA Logs

A blazingly fast DPS meter for Lost Ark, written in Rust.


LOA Logs is a "blazingly fast" open source Lost Ark DPS meter [Github](https://github.com/snoww/loa-logs) written in Rust by [Snow](https://github.com/snoww).


This project is an opinionated flavor of [LOA Details](https://github.com/lost-ark-dev/loa-details) by Herysia and Mathi, but should share very similar user interfaces and settings. The packet sniffing is still done by LOA Details' [`meter-core`](https://github.com/lost-ark-dev/meter-core) under the hood, but the data processing is done using Rust. There are future plans to port the packet sniffing part to Rust as well.