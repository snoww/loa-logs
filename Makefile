default: dep


# TODO: use these somehow to make builds faster? im the dumb and powershell isnt bash
core-vars = RUST_BACKTRACE=1 CORE_METER_MANIFEST_DIR=$$(pwd)/$(lcl-core-dir) CARGO_TARGET_DIR=$(lcl-core-dir)/target
lcl-core-dir = src-tauri\meter-core-rs

$(lcl-core-dir):
	git clone ??? $(lcl-core-dir)

deps: $(lcl-core-dir)
	$(MAKE) -C $(lcl-core-dir) deps
	npm install
	rustup target add x86_64-pc-windows-gnu
	rustup toolchain install stable-x86_64-pc-windows-gnu
	rustup default stable-x86_64-pc-windows-gnu
	cargo install -v --target x86_64-pc-windows-gnu --path .\src-tauri
