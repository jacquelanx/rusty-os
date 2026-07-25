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
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
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