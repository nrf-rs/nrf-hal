#[cfg(feature = "52832")]
pub mod target {
    pub mod uart {
        // Size of the UARTE.(TXD/RXD).MAXCOUNT register.
        pub const MAX_BUFFER_LENGTH: usize = u8::max_value() as usize;
    }
    pub mod spi {
        // Size of the SPI.(TXD/RXD).MAXCOUNT register.
        pub const MAX_BUFFER_LENGTH: usize = u8::max_value() as usize;
    }
    pub mod twi {
        // Size of the TWI.(TXD/RXD).MAXCOUNT register.
        pub const MAX_BUFFER_LENGTH: usize = u8::max_value() as usize;
    }
}

#[cfg(feature = "52840")]
pub mod target {
    pub mod uart {
        // Size of the UARTE.(TXD/RXD).MAXCOUNT register.
        pub const MAX_BUFFER_LENGTH: usize = u16::max_value() as usize;
    }
    pub mod spi {
        // Size of the SPI.(TXD/RXD).MAXCOUNT register.
        pub const MAX_BUFFER_LENGTH: usize = u16::max_value() as usize;
    }
    pub mod twi {
        // Size of the TWI.(TXD/RXD).MAXCOUNT register.
        pub const MAX_BUFFER_LENGTH: usize = u16::max_value() as usize;
    }
}