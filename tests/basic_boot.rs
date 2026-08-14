/* Integration tests are separate executables, so we need to provide all crate
attributes like no_std again and also create a new entry point function _start.
_start calls test_main, the test entry point function. */

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(rusty_os::test_runner)]


use core::panic::PanicInfo;
use rusty_os::println;


#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


/* ----------------------------- Basic Boot Tests ----------------------------- */

#[test_case]
fn test_println() {
    println!("test_println output");
}
