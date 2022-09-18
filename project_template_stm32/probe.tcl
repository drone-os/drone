### Depending on your debug probe, use one of the following config sections or
### refer to the OpenOCD documentation for the list of all supported probes:
### https://openocd.org/doc/html/Debug-Adapter-Hardware.html
###
### * ST-Link
source [find interface/stlink.cfg]
### * J-Link
# source [find interface/jlink.cfg]
# transport select swd

### Choose your target microcontroller or evaluation board config.
source [find board/stm32f103c8_blue_pill.cfg]

rename shutdown original_shutdown
proc shutdown {} {
    drone_stream stop nofail
    original_shutdown
}

### If you experience problems with your STM32 microcontroller to read Drone
### Stream, try uncommenting this fix.
# # AHB-AP doesn't work in low-power modes even with DBG_STANDBY/DBG_STOP/
# # DBG_SLEEP set. Enabling a DMA clock fixes it.
# proc ahb_ap_fix {} {
#     echo "Enabling DMA1 clock!"
#     # RCC_AHBENR |= DMA1EN
#     mmw 0x40021014 0x00000001 0
# }
# 
# rename before_drone_stream_reset original_before_drone_stream_reset
# proc before_drone_stream_reset {} {
#     ahb_ap_fix
#     original_before_drone_stream_reset
# }
# 
# rename before_drone_stream_run original_before_drone_stream_run
# proc before_drone_stream_run {} {
#     echo "Pausing for a moment to apply a fix!"
#     halt
#     ahb_ap_fix
#     resume
#     original_before_drone_stream_run
# }
