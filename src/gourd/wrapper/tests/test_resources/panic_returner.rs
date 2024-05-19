// This file does NOT belong in a module.
// It is a resource compiled independently in the unit tests for `wrapper/mod.rs`.

#![no_main]
#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
