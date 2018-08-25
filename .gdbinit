# print demangled symbols by default
set print asm-demangle on

# JLink
target extended-remote :2331
monitor flash breakpoints 1
# allow hprints to show up in gdb
monitor semihosting enable
monitor semihosting IOClient 3

monitor reset
load

# OpenOCD
#target extended-remote :3333
#monitor arm semihosting enable
# send captured ITM to the file itm.fifo
# (the microcontroller SWO pin must be connected to the programmer SWO pin)
# 8000000 must match the core clock frequency
# monitor tpiu config internal itm.fifo uart off 8000000
# OR: make the microcontroller SWO pin output compatible with UART (8N1)
# 2000000 is the frequency of the SWO pin
# monitor tpiu config external uart off 8000000 2000000
# enable ITM port 0
# monitor itm port 0 on

#load
#step
