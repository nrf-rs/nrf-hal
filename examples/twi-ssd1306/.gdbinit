set remotetimeout 60000
target remote :2331
set arm force-mode thumb

# Uncomment to enable semihosting, when necessary
monitor semihosting enable

layout split
monitor reset
load
