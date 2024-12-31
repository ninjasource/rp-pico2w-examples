# Raspberry Pi Pico2 W examples

This repo was created to demonstrate a simple network interaction between two rp pico2w dev boards with minimal external circuitry required. 

## Setup

Install / update bootloader tool:
```
cargo install elf2uf2-rs
```

Wifi connection settings:

In order to connect to the wifi network please create the following two files in the `src` folder:
`WIFI_SSID.txt` and `WIFI_PASSWORD.txt`
The files above should contain the `exact` ssid and password to connect to the wifi network. No newline characters or quotes.

## Troubleshooting

Error running:
`Error: "Unable to find mounted pico"`
Reason: The pico2 cannot be detected in bootloader mody by the host
Solution: Unplug the pico, hold down the BOOTSEL button while plugging it back in. On Linux you need to mound the usb drive (e.g. by clicking on RP2350 in your file explorer - you should see two files)

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