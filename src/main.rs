#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rusty_os::test_runner)]
#![reexport_test_harness_main = "test_main"]


use core::panic::PanicInfo;
use rusty_os::println;


#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}


/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}


#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rusty_os::test_panic_handler(info)
}






/* OLD CODE TRASHBIN. KEPT FOR LEARNING PURPOSES ONLY. 

------ Our old method for writing to VGA before we used good old OOP: ------

static HELLO: &[u8] = b"Hello World!";
/* Here, "Hello World!" is a byte string with type &[u8]. This is an uneditable
array of unsigned 8-bit integers. So essentially it looks like:
72 101 108 108 111 ...
H   e   l   l   o
Where each letter is a byte. Rust slices (aka strings) always know their length.
*/

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // buffer starts at 0xb8000; treat this as a mutable pointer of type u8 (bytes)
    let vga_buffer = 0xb8000 as *mut u8;

    // buffer stores 2 bytes per character: for char & color
    // HELLO.iter() returns &u8, &u8, &u8, ... (reference to bytes)
    for (i, &byte) in HELLO.iter().enumerate() {  // enumerate gives index
        unsafe {  // we're sure we're valid
            *vga_buffer.offset(i as isize * 2) = byte;  // in c: *(vga + i*2) = byte, aka the letter;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;  // set color to cyan
        }
    }
    loop {}
    // i miss c :sob
}

------------------------------- End of Trashbin -------------------------------
*/