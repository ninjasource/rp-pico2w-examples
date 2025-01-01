# Raspberry Pi Pico2 W examples

This repo was created to demonstrate a simple network interaction between two rp pico2w dev boards with minimal external circuitry required. Specifically we don't even need a probe (debugger).

## Setup

Install bootloader tool
See https://github.com/raspberrypi/picotool

Install inotifywait
```
sudo apt install inotify-tools
```

Wifi connection settings:

In order to connect to the wifi network please create the following two files in the `src` folder:
`WIFI_SSID.txt` and `WIFI_PASSWORD.txt`
The files above should contain the `exact` ssid and password to connect to the wifi network. No newline characters or quotes.

## Troubleshooting

Error running:
`No accessible RP-series devices in BOOTSEL mode were found.`
Reason: The pico2 cannot be detected in bootloader mody by the host
Solution: Unplug the pico, hold down the BOOTSEL button while plugging it back in. You can then release the button and the device should remain in boot mode.

Compile errors:
```
couldn't read `src/WIFI_SSID.txt`: No such file or directory (os error 2)
couldn't read `src/WIFI_PASSWORD.txt`: No such file or directory (os error 2)
```
Solution: You need to create the file above with the wifi network name in the file. No quotes required and no newlines either!

## What is the `memory.x` file? 

One of the last steps in compilation is linking which is the process of assigning physical memory addreses to variables and code.
On a computer with an operating system the OS uses virtual memory but embedded systems like the rp-pico don't have an OS 
and we need to create an executable with physical memory addresses in the correct locations that are expected by the pico. 
The `memory.x` file is the the developer facing linker script that tells the linker when RAM and FLASH physically start. 
If you look at `.cargo/config.toml` you will see a whole bunch of linker scripts referenced there. The `link.x` script references `memory.x`. 
Additionally, the `defmt.x` script is used for logging. This allows the logging mechanism to save resources by not having to format large strings of text as it simply references the debug symbols instead.

## How can this be compiled on a PC and run on a pico?

Rust supports cross compilation and this is setup in the `.cargo/config.toml` file with the following config:

```
[build]
target = "thumbv8m.main-none-eabihf" 
```

## How to run the examples

See `01_logs.rs` for instructions on how the pico bootloader works.


```
cargo run --bin 01_logs --release
```

## Programming using probe-rs

Currently, there is no mainline support for programming the RP235x in probe-rs but you can try this fork for now:

```
cargo install --git https://github.com/konkers/probe-rs --branch wip/2350 probe-rs-tools --locked
```
Note that this appears to require a probe (debugger) and cannot be used to flash via the USB bootloader

## Reading from the serial port to see the logs

```
screen /dev/ttyACM0 115200
```
