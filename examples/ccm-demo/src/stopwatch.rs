use super::hal::pac::TIMER0;

pub struct StopWatch {
    regs: TIMER0,
}

impl StopWatch {
    pub fn new(regs: TIMER0) -> Self {
        // NOTE(unsafe) 1 is a valid pattern to write to this register
        regs.tasks_stop.write(|w| unsafe { w.bits(1) });

        regs.bitmode.write(|w| w.bitmode()._32bit());

        // 16 Mhz / 2**4 = 1 Mhz = µs resolution
        // NOTE(unsafe) 4 is a valid pattern to write to this register
        regs.prescaler.write(|w| unsafe { w.prescaler().bits(4) });
        // NOTE(unsafe) 1 is a valid pattern to write to this register
        regs.tasks_clear.write(|w| unsafe { w.bits(1) });

        Self { regs }
    }

    #[inline(always)]
    pub fn start(&mut self) {
        // NOTE(unsafe) 1 is a valid pattern to write to this register
        self.regs.tasks_start.write(|w| unsafe { w.bits(1) });
    }

    #[inline(always)]
    pub fn now(&self) -> u32 {
        // NOTE(unsafe) 1 is a valid pattern to write to this register
        self.regs.tasks_capture[0].write(|w| unsafe { w.bits(1) });
        self.regs.cc[0].read().bits()
    }

    #[inline(always)]
    pub fn stop(&mut self) {
        // NOTE(unsafe) 1 is a valid pattern to write to this register
        self.regs.tasks_stop.write(|w| unsafe { w.bits(1) });

        // NOTE(unsafe) 1 is a valid pattern to write to this register
        self.regs.tasks_clear.write(|w| unsafe { w.bits(1) });
    }
}
