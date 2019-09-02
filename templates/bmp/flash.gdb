target extended-remote {{config.bmp.gdb_endpoint}}
monitor version
monitor connect_srst enable
monitor swdp_scan
attach 1
load
compare-sections
kill
