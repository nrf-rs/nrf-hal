target remote :2331
set backtrace limit 32
load
monitor reset
break main
layout split
continue
