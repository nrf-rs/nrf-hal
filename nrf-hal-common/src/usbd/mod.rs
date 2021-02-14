//! A `usb-device` implementation using the USBD peripheral.
//!
//! Difficulties:
//! * Control EP 0 is special:
//!   * Setup stage is put in registers, not RAM.
//!   * Different events are used to initiate transfers.
//!   * No notification when the status stage is ACK'd.

mod errata;

use crate::target::USBD;
use core::sync::atomic::{compiler_fence, Ordering};
use core::{cell::Cell, mem, ptr, slice};
use cortex_m::interrupt::{self, Mutex};
use usb_device::{
    bus::{PollResult, UsbBus, UsbBusAllocator},
    endpoint::{EndpointAddress, EndpointType},
    UsbDirection, UsbError,
};

fn dma_start() {
    compiler_fence(Ordering::Release);
}

fn dma_end() {
    compiler_fence(Ordering::Acquire);
}

struct Buffers {
    // Ptr and size is stored separately to save space
    in_bufs: [*mut u8; 9],
    out_bufs: [*mut u8; 9],

    // Buffers can be up to 64 Bytes since this is a Full-Speed implementation.
    in_lens: [u8; 9],
    out_lens: [u8; 9],
}

impl Buffers {
    fn new() -> Self {
        Self {
            in_bufs: [ptr::null_mut(); 9],
            out_bufs: [ptr::null_mut(); 9],
            in_lens: [0; 9],
            out_lens: [0; 9],
        }
    }
}

unsafe impl Sync for Buffers {}

/// USB device implementation.
pub struct Usbd {
    periph: Mutex<USBD>,
    unalloc_buffers: &'static mut [u8],
    bufs: Buffers,
    used_in: u8,
    used_out: u8,
    iso_in_used: bool,
    iso_out_used: bool,
    // FIXME: The used flags should probably be collapsed into u16's.
    /// Keeps track of which IN (device -> host) buffers are currently in use.
    ///
    /// The bits for every IN EP are set in `write` and remain 1 until `poll` detects that the write
    /// is finished (or at least DMA from the buffer has finished). While the bit is 1, `write`
    /// returns `WouldBlock`.
    in_bufs_in_use: Mutex<Cell<u16>>,
}

impl Usbd {
    /// Creates a new USB bus, taking ownership of the raw peripheral.
    ///
    /// # Parameters
    ///
    /// * `periph`: The raw USBD peripheral.
    /// * `endpoint_buffers`: Backing storage for the endpoint buffers. This
    ///   needs to be big enough to accomodate all buffers of all endpoints, or
    ///   `alloc_ep` will fail.
    #[inline]
    pub fn new_alloc(periph: USBD, endpoint_buffers: &'static mut [u8]) -> UsbBusAllocator<Self> {
        UsbBusAllocator::new(Self {
            periph: Mutex::new(periph),
            unalloc_buffers: endpoint_buffers,
            bufs: Buffers::new(),
            used_in: 0,
            used_out: 0,
            iso_in_used: false,
            iso_out_used: false,
            in_bufs_in_use: Mutex::new(Cell::new(0)),
        })
    }

    /// Fetches the address assigned to the device (only valid when device is configured).
    pub fn device_address(&self) -> u8 {
        unsafe { &*USBD::ptr() }.usbaddr.read().addr().bits()
    }

    fn alloc_ep_buf(
        &mut self,
        ep_type: EndpointType,
        mut size: u16,
    ) -> usb_device::Result<&'static mut [u8]> {
        if self.unalloc_buffers.len() < usize::from(size) {
            Err(UsbError::EndpointMemoryOverflow)
        } else {
            if ep_type == EndpointType::Bulk || ep_type == EndpointType::Interrupt {
                // datasheet: buffer must be 4-byte aligned and its size must be a multiple of 4
                let rem = self.unalloc_buffers.as_mut_ptr() as usize % 4;
                if rem != 0 {
                    let (_padding, remaining) =
                        mem::replace(&mut self.unalloc_buffers, &mut []).split_at_mut(4 - rem);
                    self.unalloc_buffers = remaining;
                }

                let rem = size % 4;
                if rem != 0 {
                    size = size + 4 - rem;
                }
            }
            assert!(size <= 64);
            let (alloc, remaining) =
                mem::replace(&mut self.unalloc_buffers, &mut []).split_at_mut(size.into());
            self.unalloc_buffers = remaining;
            Ok(alloc)
        }
    }

    fn is_used(&self, ep: EndpointAddress) -> bool {
        if ep.index() == 8 {
            // ISO
            let flag = if ep.is_in() {
                self.iso_in_used
            } else {
                self.iso_out_used
            };
            return flag;
        }

        if ep.is_in() {
            self.used_in & (1 << ep.index()) != 0
        } else {
            self.used_out & (1 << ep.index()) != 0
        }
    }

    fn read_control_setup(&self, regs: &USBD, buf: &mut [u8]) -> usb_device::Result<usize> {
        const SETUP_LEN: usize = 8;

        if buf.len() < SETUP_LEN {
            return Err(UsbError::BufferOverflow);
        }

        // This is unfortunate: Reassemble the nicely split-up setup packet back into bytes, only
        // for the usb-device code to copy the bytes back out into structured data.
        // The bytes are split across registers, leaving a 3-Byte gap between them, so we couldn't
        // even memcpy all of them at once. Weird peripheral.
        buf[0] = regs.bmrequesttype.read().bits() as u8;
        buf[1] = regs.brequest.read().brequest().bits();
        buf[2] = regs.wvaluel.read().wvaluel().bits();
        buf[3] = regs.wvalueh.read().wvalueh().bits();
        buf[4] = regs.windexl.read().windexl().bits();
        buf[5] = regs.windexh.read().windexh().bits();
        buf[6] = regs.wlengthl.read().wlengthl().bits();
        buf[7] = regs.wlengthh.read().wlengthh().bits();

        if regs.bmrequesttype.read().direction().is_host_to_device() {
            let ptr = self.bufs.out_bufs[0];
            let len = self.bufs.out_lens[0];

            unsafe {
                regs.epout0.ptr.write(|w| w.bits(ptr as u32));
                regs.epout0.maxcnt.write(|w| w.bits(u32::from(len)));
            }
            regs.tasks_ep0rcvout
                .write(|w| w.tasks_ep0rcvout().set_bit());
        }

        Ok(SETUP_LEN)
    }
}

impl UsbBus for Usbd {
    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> usb_device::Result<EndpointAddress> {
        // Endpoint addresses are fixed in hardware:
        // - 0x80 / 0x00 - Control        EP0
        // - 0x81 / 0x01 - Bulk/Interrupt EP1
        // - 0x82 / 0x02 - Bulk/Interrupt EP2
        // - 0x83 / 0x03 - Bulk/Interrupt EP3
        // - 0x84 / 0x04 - Bulk/Interrupt EP4
        // - 0x85 / 0x05 - Bulk/Interrupt EP5
        // - 0x86 / 0x06 - Bulk/Interrupt EP6
        // - 0x87 / 0x07 - Bulk/Interrupt EP7
        // - 0x88 / 0x08 - Isochronous

        // Endpoint directions are allocated individually.

        let buf = self.alloc_ep_buf(ep_type, max_packet_size)?;

        if false {
            unimplemented!(
                "alloc_ep({:?}, {:?}, {:?}, {}, {})",
                ep_dir,
                ep_addr,
                ep_type,
                max_packet_size,
                interval,
            );
        }

        let (used, bufs, lens) = match ep_dir {
            UsbDirection::In => (
                &mut self.used_in,
                &mut self.bufs.in_bufs,
                &mut self.bufs.in_lens,
            ),
            UsbDirection::Out => (
                &mut self.used_out,
                &mut self.bufs.out_bufs,
                &mut self.bufs.out_lens,
            ),
        };

        let alloc_index = match ep_type {
            EndpointType::Isochronous => {
                let flag = match ep_dir {
                    UsbDirection::In => &mut self.iso_in_used,
                    UsbDirection::Out => &mut self.iso_out_used,
                };

                if *flag {
                    return Err(UsbError::EndpointOverflow);
                } else {
                    *flag = true;
                    bufs[8] = buf.as_mut_ptr();
                    lens[8] = buf.len() as u8;
                    return Ok(EndpointAddress::from_parts(0x08, ep_dir));
                }
            }
            EndpointType::Control => 0,
            EndpointType::Interrupt | EndpointType::Bulk => {
                let leading = used.leading_zeros();
                if leading == 0 {
                    return Err(UsbError::EndpointOverflow);
                }

                if leading == 8 {
                    // Even CONTROL is free, don't allocate that
                    1
                } else {
                    8 - leading
                }
            }
        };

        if *used & (1 << alloc_index) != 0 {
            return Err(UsbError::EndpointOverflow);
        }

        *used |= 1 << alloc_index;
        bufs[alloc_index as usize] = buf.as_mut_ptr();
        lens[alloc_index as usize] = buf.len() as u8;

        let addr = EndpointAddress::from_parts(alloc_index as usize, ep_dir);
        Ok(addr)
    }

    #[inline]
    fn enable(&mut self) {
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);

            errata::pre_enable();

            regs.enable.write(|w| w.enable().enabled());

            // Wait until the peripheral is ready.
            while !regs.eventcause.read().ready().is_ready() {}
            regs.eventcause.write(|w| w.ready().set_bit()); // Write 1 to clear.

            errata::post_enable();

            // Enable the USB pullup, allowing enumeration.
            regs.usbpullup.write(|w| w.connect().enabled());
        });
    }

    #[inline]
    fn reset(&self) {
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);

            // Neat little hack to work around svd2rust not coalescing these on its own.
            let epin = [
                &regs.epin0,
                &regs.epin1,
                &regs.epin2,
                &regs.epin3,
                &regs.epin4,
                &regs.epin5,
                &regs.epin6,
                &regs.epin7,
            ];
            let epout = [
                &regs.epout0,
                &regs.epout1,
                &regs.epout2,
                &regs.epout3,
                &regs.epout4,
                &regs.epout5,
                &regs.epout6,
                &regs.epout7,
            ];

            // TODO: Initialize ISO buffers
            // Initialize all data pointers for the endpoint buffers, since they never change.
            for i in 0..8 {
                let in_enabled = self.used_in & (1 << i) != 0;
                let out_enabled = self.used_out & (1 << i) != 0;

                if in_enabled {
                    unsafe {
                        epin[i].ptr.write(|w| w.bits(self.bufs.in_bufs[i] as u32));
                        epin[i]
                            .maxcnt
                            .write(|w| w.bits(u32::from(self.bufs.in_lens[i])));
                    }
                }

                if out_enabled {
                    unsafe {
                        epout[i].ptr.write(|w| w.bits(self.bufs.out_bufs[i] as u32));
                        epout[i]
                            .maxcnt
                            .write(|w| w.bits(u32::from(self.bufs.out_lens[i])));
                    }
                }
            }

            // TODO: Merge `used_{in,out}` with `iso_{in,out}_used` so ISO is enabled here as well.
            // Make the enabled endpoints respond to traffic.
            unsafe {
                regs.epinen.write(|w| w.bits(self.used_in.into()));
                regs.epouten.write(|w| w.bits(self.used_out.into()));
            }
        });
    }

    #[inline]
    fn set_device_address(&self, _addr: u8) {
        // Nothing to do, the peripheral handles this.
    }

    fn write(&self, ep_addr: EndpointAddress, buf: &[u8]) -> usb_device::Result<usize> {
        if !self.is_used(ep_addr) {
            return Err(UsbError::InvalidEndpoint);
        }

        if ep_addr.is_out() {
            return Err(UsbError::InvalidEndpoint);
        }

        // A 0-length write to Control EP 0 is a status stage acknowledging a control write xfer
        if ep_addr.index() == 0 && buf.is_empty() {
            // There is no need to mark the buffer as used here.

            interrupt::free(|cs| {
                let regs = self.periph.borrow(cs);
                regs.tasks_ep0status
                    .write(|w| w.tasks_ep0status().set_bit());
            });
            // XXX anything else to do?
            return Ok(0);
        }

        let i = ep_addr.index();

        if usize::from(self.bufs.in_lens[i]) < buf.len() {
            return Err(UsbError::BufferOverflow);
        }

        interrupt::free(|cs| {
            let in_bufs_in_use = self.in_bufs_in_use.borrow(cs);

            // We cannot use the ENDEPIN event to detect whether it's fine to write to the buffer,
            // since that is not active when first initializing the peripheral. It also gets cleared
            // when `poll`ed, and thus would be unavailable here.
            // We store an additional bitflags in the peripheral wrapper to store which IN buffers are
            // in use by DMA, and which may be written to.
            if in_bufs_in_use.get() & (1 << i) != 0 {
                return Err(UsbError::WouldBlock);
            }

            // Mark buffer as "in use".
            in_bufs_in_use.set(in_bufs_in_use.get() | (1 << i));

            Ok(())
        })?;

        // Now the target buffer is locked and we can copy the data outside of the critical
        // sections. If we get interrupted, the buffer can't be acquired (and the code that would
        // *unlock* the buffer should never be invoked here anyways).
        let ptr = self.bufs.in_bufs[i];
        let len = self.bufs.in_lens[i];
        let slice = unsafe { slice::from_raw_parts_mut(ptr, usize::from(len)) };
        slice[..buf.len()].copy_from_slice(buf);

        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);

            let epin = [
                &regs.epin0,
                &regs.epin1,
                &regs.epin2,
                &regs.epin3,
                &regs.epin4,
                &regs.epin5,
                &regs.epin6,
                &regs.epin7,
            ];

            // Set the buffer length so the right number of bytes are transmitted.
            // Safety: `buf.len()` has been checked to be <= the max buffer length.
            unsafe {
                epin[i].maxcnt.write(|w| w.maxcnt().bits(buf.len() as u8));
                epin[i].ptr.write(|w| w.bits(self.bufs.in_bufs[i] as u32));
            }

            // Kick off device -> host transmission. This starts DMA, so a compiler fence is needed.
            dma_start();
            regs.tasks_startepin[i].write(|w| w.tasks_startepin().set_bit());

            Ok(buf.len())
        })
    }

    fn read(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> usb_device::Result<usize> {
        if !self.is_used(ep_addr) {
            return Err(UsbError::InvalidEndpoint);
        }

        if ep_addr.is_in() {
            return Err(UsbError::InvalidEndpoint);
        }

        let i = ep_addr.index();
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);

            // Control EP 0 is special
            if i == 0 {
                // Control setup packet is special, since it is put in registers, not a buffer.
                if regs.events_ep0setup.read().events_ep0setup().bit_is_set() {
                    regs.events_ep0setup.reset();
                    return self.read_control_setup(regs, buf);
                } else {
                    // XXX hack!
                    regs.tasks_ep0status
                        .write(|w| w.tasks_ep0status().set_bit());
                    return Ok(0);
                }
            }

            // Is the endpoint ready? (ie. has DMA finished?)
            // TODO: ISO
            if !regs.events_endepout[i]
                .read()
                .events_endepout()
                .bit_is_set()
            {
                // Not yet ready.
                return Err(UsbError::WouldBlock);
            }

            regs.events_endepout[i].reset();

            // How much was transferred?
            let len = regs.size.epout[i].read().size().bits();

            if usize::from(len) > buf.len() {
                return Err(UsbError::BufferOverflow);
            }

            // Make sure it's smaller than the buffer size.
            let bufsz = self.bufs.out_lens[i];
            assert!(len <= bufsz);

            let ptr = self.bufs.out_bufs[i];

            // Safety: `len` is in bounds and DMA is not writing to the buffer.
            let slice = unsafe { slice::from_raw_parts(ptr, usize::from(len)) };
            buf[..usize::from(len)].copy_from_slice(slice);

            // Done copying. Now we need to allow the EP to receive the next packet (ie. clear NAK).
            // This is done by writing anything to `SIZE.EPOUT[i]`.
            // Safety note: This effectively starts DMA, so we need a corresponding barrier.
            dma_start();
            regs.size.epout[i].reset();

            Ok(usize::from(len))
        })
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);

            unsafe {
                if ep_addr.index() == 0 {
                    regs.tasks_ep0stall.write(|w| w.tasks_ep0stall().set_bit());
                } else {
                    regs.epstall.write(|w| {
                        w.ep()
                            .bits(ep_addr.index() as u8 & 0b111)
                            .io()
                            .bit(ep_addr.is_in())
                            .stall()
                            .bit(stalled)
                    });
                }
            }
        });
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        unimplemented!("is_stalled(ep={:?})", ep_addr);
    }

    #[inline]
    fn suspend(&self) {
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);
            regs.lowpower.write(|w| w.lowpower().low_power());
        });
    }

    #[inline]
    fn resume(&self) {
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);

            errata::pre_wakeup();

            regs.lowpower.write(|w| w.lowpower().force_normal());
            
        });
    }

    fn poll(&self) -> PollResult {
        interrupt::free(|cs| {
            let in_bufs_in_use = self.in_bufs_in_use.borrow(cs);
            let regs = self.periph.borrow(cs);

            if regs.events_usbreset.read().events_usbreset().bit_is_set() {
                regs.events_usbreset.reset();
                return PollResult::Reset;
            } else if regs.events_usbevent.read().events_usbevent().bit_is_set() {
                // "Write 1 to clear"
                if regs.eventcause.read().suspend().bit() {
                    regs.eventcause.write(|w| w.suspend().bit(true));
                    return PollResult::Suspend;
                } else if regs.eventcause.read().resume().bit() {
                    regs.eventcause.write(|w| w.resume().bit(true));
                    return PollResult::Resume;
                } else {
                    regs.events_usbevent.reset();
                }
            }

            // Check for any finished transmissions.
            let mut in_complete = 0;
            let mut out_complete = 0;
            for i in 0..=7 {
                if i == 0 {
                    if regs
                        .events_ep0datadone
                        .read()
                        .events_ep0datadone()
                        .bit_is_set()
                    {
                        dma_end();

                        // Clear event, since we must only report this once.
                        regs.events_ep0datadone.reset();
                        in_complete |= 1 << i;

                        // The associated buffer is free again.
                        in_bufs_in_use.set(in_bufs_in_use.get() & !(1 << i));
                    }
                } else {
                    if regs.events_endepin[i].read().events_endepin().bit_is_set() {
                        dma_end();

                        // Clear event, since we must only report this once.
                        regs.events_endepin[i].reset();
                        in_complete |= 1 << i;

                        // The associated buffer is free again.
                        in_bufs_in_use.set(in_bufs_in_use.get() & !(1 << i));
                    }
                }

                if regs.events_endepout[i]
                    .read()
                    .events_endepout()
                    .bit_is_set()
                {
                    // Do not clear OUT events, since we have to continue reporting them until the
                    // buffer is read.
                    out_complete |= 1 << i;
                }
            }

            // Setup packets are only relevant on the control EP 0.
            let mut ep_setup = 0;
            if regs.events_ep0setup.read().events_ep0setup().bit_is_set() {
                ep_setup = 1;
            }

            // TODO: Check ISO EP

            if out_complete != 0 || in_complete != 0 || ep_setup != 0 {
                PollResult::Data {
                    ep_out: out_complete,
                    ep_in_complete: in_complete,
                    ep_setup,
                }
            } else {
                PollResult::None
            }
        })
    }

    fn force_reset(&self) -> usb_device::Result<()> {
        interrupt::free(|cs| {
            let regs = self.periph.borrow(cs);
            regs.usbpullup.write(|w| w.connect().disabled());
            // TODO delay needed?
            regs.usbpullup.write(|w| w.connect().enabled());
        });

        Ok(())
    }

    /// The peripheral handles this for us.
    ///
    /// The Reference Manual says:
    ///
    /// > Note: The USBD peripheral handles the SetAddress transfer by itself. As a consequence, the
    /// > software shall not process this command other than updating its state machine (see Device
    /// > state diagram), nor initiate a status stage. If necessary, the address assigned by the
    /// > host can be read out from the USBADDR register after the command has been processed.
    const INHIBIT_SET_ADDRESS_RESPONSE: bool = true;
}
