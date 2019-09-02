target extended-remote {{config.bmp.gdb_endpoint}}
monitor version
{{#if reset}}
monitor connect_srst enable
{{/if}}
monitor swdp_scan
attach 1
set mem inaccessible-by-default off
{{#bmp-targets "stm32f100"
               "stm32f101"
               "stm32f102"
               "stm32f103"
               "stm32f107"
               "stm32l4x1"
               "stm32l4x2"
               "stm32l4x3"
               "stm32l4x5"
               "stm32l4x6"
               "stm32l4r5"
               "stm32l4r7"
               "stm32l4r9"
               "stm32l4s5"
               "stm32l4s7"
               "stm32l4s9" }}
{{> stm32 }}
set {int}$DBGMCU_CR = {int}$DBGMCU_CR | $DBGMCU_CR_DBG_STANDBY | $DBGMCU_CR_DBG_STOP | $DBGMCU_CR_DBG_SLEEP
{{/bmp-targets}}
