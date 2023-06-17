//! Implementation of the 1-Wire protocol.
//!
//! [1-Wire](https://www.maximintegrated.com/en/design/technical-documents/app-notes/1/126.html)

#![no_std]
#![feature(decl_macro)]
#![feature(default_free_fn)]
#![feature(error_in_core)]
#![feature(trait_alias)]

pub use code::Code;
pub use command::{Command, Commander};
pub use error::{Error, Result};

use embedded_hal::{
    delay::DelayUs,
    digital::{ErrorType, InputPin, OutputPin},
};
use embedded_hal_async::digital::Wait;
use standard::*;

/// 1 Wire
#[derive(Clone, Copy, Debug, Default)]
pub struct OneWire<T, U> {
    delay: U,
    pin: T,
    speed: Speed,
}

impl<T: InputPin + ErrorType, U> OneWire<T, U> {
    pub fn is_high(&self) -> Result<bool, T::Error> {
        Ok(self.pin.is_high()?)
    }

    pub fn is_low(&self) -> Result<bool, T::Error> {
        Ok(self.pin.is_low()?)
    }
}

impl<T: OutputPin + ErrorType, U> OneWire<T, U> {
    pub fn new(pin: T, delay: U) -> Result<Self, T::Error> {
        let mut one_wire = Self {
            pin,
            delay,
            speed: Speed::Standard,
        };
        // Pin should be high during idle.
        one_wire.set_high()?;
        Ok(one_wire)
    }

    /// Set the output as high.
    ///
    /// Disconnects the bus, letting another device (or the pull-up resistor)
    pub fn set_high(&mut self) -> Result<(), T::Error> {
        Ok(self.pin.set_high()?)
    }

    /// Set the output as low.
    pub fn set_low(&mut self) -> Result<(), T::Error> {
        Ok(self.pin.set_low()?)
    }
}

impl<T, U: DelayUs> OneWire<T, U> {
    pub fn wait(&mut self, us: u32) {
        self.delay.delay_us(us);
    }
}

/// Bit (basic) operations
impl<T: InputPin + OutputPin + ErrorType, U: DelayUs> OneWire<T, U> {
    // Generate a 1-Wire reset, return true if no presence detect was found,
    // return false otherwise.
    pub fn reset(&mut self) -> Result<bool, T::Error> {
        self.wait(G);
        self.set_low()?;
        self.wait(H);
        self.set_high()?;
        self.wait(I);
        let presence = self.is_low()?;
        self.wait(J);
        Ok(presence)
    }

    /// Read a bit from the 1-Wire bus and return it. Provide 10us recovery
    /// time.
    pub fn read_bit(&mut self) -> Result<bool, T::Error> {
        self.set_low()?;
        self.wait(A);
        self.set_high()?;
        self.wait(E);
        let bit = self.is_high()?;
        self.wait(F);
        Ok(bit)
    }

    /// Send a 1-Wire write bit. Provide 10us recovery time.
    pub fn write_bit(&mut self, bit: bool) -> Result<(), T::Error> {
        if bit {
            self.write_bit_1()
        } else {
            self.write_bit_0()
        }
    }

    /// Write '0' bit.
    pub fn write_bit_0(&mut self) -> Result<(), T::Error> {
        self.set_low()?;
        self.wait(C);
        self.set_high()?;
        self.wait(D);
        Ok(())
    }

    /// Write '1' bit.
    pub fn write_bit_1(&mut self) -> Result<(), T::Error> {
        self.set_low()?;
        self.wait(A);
        self.set_high()?;
        self.wait(B);
        Ok(())
    }
}

/// Byte operations
impl<T: InputPin + OutputPin + ErrorType, U: DelayUs> OneWire<T, U> {
    /// Read 1-Wire data byte.
    pub fn read_byte(&mut self) -> Result<u8, T::Error> {
        let mut byte = 0;
        for _ in 0..u8::BITS {
            byte >>= 1;
            if self.read_bit()? {
                byte |= 0x80;
            }
        }
        Ok(byte)
    }

    pub fn read_bytes(&mut self, bytes: &mut [u8]) -> Result<(), T::Error> {
        for byte in bytes {
            *byte = self.read_byte()?;
        }
        Ok(())
    }

    /// Write 1-Wire data byte.
    pub fn write_byte(&mut self, mut byte: u8) -> Result<(), T::Error> {
        for _ in 0..u8::BITS {
            self.write_bit(byte & 0x01 == 0x01)?;
            byte >>= 1;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), T::Error> {
        for byte in bytes {
            self.write_byte(*byte)?;
        }
        Ok(())
    }
}

/// Speed
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Speed {
    #[default]
    Standard,
    Overdrive,
}

mod standard {
    pub(super) const A: u32 = 6;
    pub(super) const B: u32 = 64;
    pub(super) const C: u32 = 60;
    pub(super) const D: u32 = 10;
    pub(super) const E: u32 = 9;
    pub(super) const F: u32 = 55;
    pub(super) const G: u32 = 0;
    pub(super) const H: u32 = 480;
    pub(super) const I: u32 = 70;
    pub(super) const J: u32 = 410;
}

mod overdrive {
    pub(super) const A: f32 = 1.0;
    pub(super) const B: f32 = 7.5;
    pub(super) const C: f32 = 7.5;
    pub(super) const D: f32 = 2.5;
    pub(super) const E: f32 = 1.0;
    pub(super) const F: f32 = 7.0;
    pub(super) const G: f32 = 2.5;
    pub(super) const H: f32 = 70.0;
    pub(super) const I: f32 = 8.5;
    pub(super) const J: f32 = 40.0;
}

pub mod commands;
pub mod crc8;

mod code;
mod command;
mod error;
