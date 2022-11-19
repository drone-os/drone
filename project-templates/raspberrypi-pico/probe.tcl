### Depending on your debug probe, use one of the following config sections or
### refer to the OpenOCD documentation for the list of all supported probes:
### https://openocd.org/doc/html/Debug-Adapter-Hardware.html
###
### * Picoprobe
source [find interface/picoprobe.cfg]
### * Raspberry Pi's GPIO
# source [find interface/raspberrypi-swd.cfg]

### Choose your target microcontroller or evaluation board config.
source [find target/rp2040.cfg]

rename shutdown original_shutdown
proc shutdown {} {
    drone_stream stop nofail
    original_shutdown
}
