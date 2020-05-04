/// Writes `val` to `addr`. Used to apply Errata workarounds.
unsafe fn poke(addr: u32, val: u32) {
    (addr as *mut u32).write_volatile(val);
}

/// Reads 32 bits from `addr`.
unsafe fn peek(addr: u32) -> u32 {
    (addr as *mut u32).read_volatile()
}

pub fn pre_enable() {
    // Works around Erratum 187 on chip revisions 1 and 2.
    unsafe {
        poke(0x4006EC00, 0x00009375);
        poke(0x4006ED14, 0x00000003);
        poke(0x4006EC00, 0x00009375);
    }

    pre_wakeup();
}

pub fn post_enable() {
    post_wakeup();

    // Works around Erratum 187 on chip revisions 1 and 2.
    unsafe {
        poke(0x4006EC00, 0x00009375);
        poke(0x4006ED14, 0x00000000);
        poke(0x4006EC00, 0x00009375);
    }
}

pub fn pre_wakeup() {
    // Works around Erratum 171 on chip revisions 1 and 2.

    unsafe {
        if peek(0x4006EC00) == 0x00000000 {
            poke(0x4006EC00, 0x00009375);
        }

        poke(0x4006EC14, 0x000000C0);
        poke(0x4006EC00, 0x00009375);
    }
}

pub fn post_wakeup() {
    // Works around Erratum 171 on chip revisions 1 and 2.

    unsafe {
        if peek(0x4006EC00) == 0x00000000 {
            poke(0x4006EC00, 0x00009375);
        }

        poke(0x4006EC14, 0x00000000);
        poke(0x4006EC00, 0x00009375);
    }
}
