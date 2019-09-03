target extended-remote {{config.bmp.gdb_endpoint}}
monitor version
{{#if reset}}
monitor connect_srst enable
{{/if}}
monitor swdp_scan
attach 1
set mem inaccessible-by-default off
{{#bmp-devices "stm32f100"
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
{{> bmp/cortex_m.gdb }}
{{> bmp/stm32.gdb }}
set {int}$DBGMCU_CR = ({int}$DBGMCU_CR & ~$DBGMCU_CR_TRACE_MODE_MASK) | $DBGMCU_CR_TRACE_IOEN | \
    $DBGMCU_CR_DBG_STANDBY | $DBGMCU_CR_DBG_STOP | $DBGMCU_CR_DBG_SLEEP
set {int}$TPIU_ACPR = $DEFAULT_HCLK / {{config.bmp.uart_baudrate}} - 1
set {int}$TPIU_SPPR = $TPIU_SPPR_NRZ
set {int}$TPIU_FFCR = {int}$TPIU_FFCR & ~$TPIU_FFCR_ENFCONT
set {int}$ITM_LAR = $ITM_LAR_CODE
set {int}$ITM_TCR = ({int}$ITM_TCR & ~( \
        $ITM_TCR_TSENA | $ITM_TCR_TXENA | $ITM_TCR_SWOENA | $ITM_TCR_TSPRESCALE_MASK | \
        $ITM_TCR_GTSFREQ_MASK | $ITM_TCR_TRACE_BUS_ID_MASK \
    )) | $ITM_TCR_SYNCENA | $ITM_TCR_ITMENA | (1 << $ITM_TCR_TRACE_BUS_ID_OFFSET)
set {int}$ITM_TPR = 0
set {int}$ITM_TER0 = 0{{#each ports}} | (1 << {{this}}){{/each}}
set {int}$DWT_CTRL = ({int}$DWT_CTRL & ~$DWT_CTRL_SYNCTAP_MASK) | $DWT_CTRL_CYCCNTENA | $DWT_CTRL_SYNCTAP_26
set {int}$DWT_CYCCNT = 0xFFFFFFFF
{{/bmp-devices}}
shell echo -n "1" > {{pipe}}
shell cat {{pipe}} > /dev/null
continue
detach
