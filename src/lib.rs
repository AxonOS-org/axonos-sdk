//! AxonOS Public SDK
//! 
//! This crate provides the foundational data structures and FFI bindings 
//! required to interface hardware (BCI gateways) with the AxonOS kernel.

#![no_std] // Ensures the crate can run on bare-metal environments (without the standard library)
extern crate alloc;

pub mod telemetry;
pub mod ffi;

/// A marker trait for any hardware that can stream neural data into AxonOS.
pub trait BciStream {
    type Error;
    
    /// Starts the continuous polling of sensor data.
    fn begin_stream(&mut self) -> Result<(), Self::Error>;
}
