# disable "are you sure you want to quit?"
define hook-quit
    set confirm off
end

target remote :3333

# print demangled symbols by default
set print asm-demangle on

monitor arm semihosting enable
load
cont
