# Raspberry Pi Pico2 W examples

This Rust repo was created to demonstrate a simple network interaction between two rp pico2w dev boards (RP2350) with minimal external circuitry required. Specifically we don't even need a probe (debugger).

## Setup

Install Rust on your machine.

Add the armv8 cross compiler to the rust toolchain
```bash
rustup target add thumbv8m.main-none-eabihf
```

Install the `elf2uf2-rs` tool to help you flash the pico. This tool has been modified to support RP2350 microcontrollers.
```bash
cargo install --git https://github.com/ninjasource/elf2uf2-rs.git --branch pico2-support --force
```

## How to run the examples

See `01_logs.rs` for instructions on how the pico bootloader works.

```bash
cargo run --bin 01_logs --release
```

## Troubleshooting

Error running: `Error: "Unable to find mounted pico"`

Reason: The pico2 cannot be detected in bootloader mode by the host.

Solution: Unplug the pico, hold down the BOOTSEL button while plugging it back in. You can then release the button and the device should remain in boot mode. You may need to mount the drive if your machine does not automatically do so already.

Error accessing serial port on linux

Reason: `libudev` needs to be installed (see `serialport` crate for more)

Solution: 
```bash
sudo apt install libudev-dev
```

Reason: your serial port needs root access.

Error: After `Found pico serial on /dev/ttyACM0` message nothing happens and the program eventually terminates.

Solution: look up how to access your serial port without root on your distro

## What is the `memory.x` file? 

One of the last steps in compilation is linking which is the process of assigning physical memory addresses to variables and code.
On a computer with an operating system the OS uses virtual memory but embedded systems like the rp-pico don't have an OS 
and we need to create an executable with physical memory addresses in the correct locations that are expected by the pico. 
The `memory.x` file is the the developer facing linker script that tells the linker when RAM and FLASH physically start. 
If you look at `.cargo/config.toml` you will see a whole bunch of linker scripts referenced there. The `link.x` script references `memory.x`. 

## How can this be compiled on a PC and run on a pico?

Rust supports cross compilation and this is setup in the `.cargo/config.toml` file with the following config:

```toml
[build]
target = "thumbv8m.main-none-eabihf" 
```


## Send test data to the pico2w

In Linux (using netcat to fire and forget ipv4 udp packet):
```bash
echo -n "on" | nc -4u -w0 192.168.1.100 47900
echo -n "off" | nc -4u -w0 192.168.1.100 47900
```