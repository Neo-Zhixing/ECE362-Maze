build:
	cargo build

debug:
	cargo run

flash: build
	arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/debug/mini-proj target/thumbv6m-none-eabi/debug/mini-proj.bin
	st-flash erase	
	st-flash write target/thumbv6m-none-eabi/debug/mini-proj.bin 0x08000000
