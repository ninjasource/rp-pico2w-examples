#!/bin/bash
set -e

~/source/picotool/build/picotool load -u -v -x -t elf $1

USB_SERIAL="/dev/ttyACM0"

echo "Waiting to connect to usb serial $USB_SERIAL"
# sudo apt install inotify-tools
inotifywait  -e create,moved_to,attrib --include $USB_SERIAL -qq /dev

# press 'Ctrl-a d' to quit
screen $USB_SERIAL 115200