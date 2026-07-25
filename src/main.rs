#![no_std]  // don't link Rust standard library
#![no_main]  // disable all Rust-level entry points

/* Storytime... core contains the pieces of Rust that don't depend on an OS;
STD is built on core. Inside the core crate is a module called panic, which
contains a struct called PanicInfo. The use keyword is for ease of importing.
*/
use core::panic::PanicInfo;


/* Normally, runtime procedures must happen before the main function is called
(eg. C runtime libraries, which invoke the entry point of the Rust runtime marked
by the _start item, which calls the main function). We don't have access to that
so we need to define our own entry point. The linker looks for a function called
`_start` by default.
*/
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


/* Since we removed the STD, we need to define the panic handler ourselves.
The panic handler is the function that the compiler calls when a panic occurs.
A panic is a Bad Thing (an unrecoverable error, like indexing out of bounds.)
Here, the PanicInfo parameter is an object that contains the file and line 
where the panic happened and an optional panic message. This panic() function 
should never return so it returns the "never" type !. Note it's not return
NOTHING, it's NEVER return.
*/ 
#[panic_handler]  // this is an attribute; it tells the compiler this is special
// the underscore = "ik this variable exists but im not using it"
// the & means you've recieved a reference to PanicInfo through its address
// HOWEVER the & is NOT &mut, so you can't modify PanicInfo
fn panic(_info: &PanicInfo) -> ! { 
    loop {}  // rip long comments #learning
}