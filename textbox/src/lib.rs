#[macro_use]
extern crate bitflags;
extern crate bit_set;

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::*;
