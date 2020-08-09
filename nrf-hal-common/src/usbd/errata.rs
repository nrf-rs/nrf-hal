/// Writes `val` to `addr`. Used to apply Errata workarounds.
pub unsafe fn poke(addr: u32, val: u32) {
    (addr as *mut u32).write_volatile(val);
}

/// Reads 32 bits from `addr`.
pub unsafe fn peek(addr: u32) -> u32 {
    (addr as *mut u32).read_volatile()
}

pub fn pre_enable() {
    // Works around Erratum 187 on chip revisions 1 and 2.
    unsafe {
        if peek(0x4006EC00) == 0x0000_0000 {
            poke(0x4006EC00, 0x0000_9375);
            poke(0x4006ED14, 0x0000_0003);
            poke(0x4006EC00, 0x0000_9375);
        } else {
            poke(0x4006ED14, 0x0000_0003);
        }
    }

    pre_wakeup();
}

pub fn post_enable() {
    // post_wakeup();

    // Works around Erratum 187 on chip revisions 1 and 2.
    unsafe {
        if peek(0x4006EC00) == 0x0000_0000 {
            poke(0x4006EC00, 0x0000_9375);
            poke(0x4006ED14, 0x0000_0000);
            poke(0x4006EC00, 0x0000_9375);
        } else {
            poke(0x4006ED14, 0x0000_0000);
        }
    }
}

pub fn pre_wakeup() {
    unsafe {
        // Works around Erratum 171 on chip revisions 1 and 2.
        if peek(0x4006EC00) == 0x0000_0000 {
            poke(0x4006EC00, 0x0000_9375);
            poke(0x4006EC14, 0x0000_00C0);
            poke(0x4006EC00, 0x0000_9375);
        } else {
            poke(0x4006EC14, 0x0000_00C0);
        }
    }
}

pub fn post_wakeup() {
    // Works around Erratum 171 on chip revisions 1 and 2.

    unsafe {
        if peek(0x4006EC00) == 0x0000_0000 {
            poke(0x4006EC00, 0x0000_9375);
            poke(0x4006EC14, 0x0000_0000);
            poke(0x4006EC00, 0x0000_9375);
        } else {
            poke(0x4006EC14, 0x0000_0000);
        }
    }
}

pub fn dma_pending_set() {
    unsafe {
        poke(0x40027C1C, 0x0000_0082);
    }
}

pub fn dma_pending_clear() {
    unsafe {
        poke(0x40027C1C, 0x0000_0000);
    }
}
