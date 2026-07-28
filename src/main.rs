#![no_std]  // don't link Rust standard library
#![no_main]  // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]


mod vga_buffer;  // include this module in this crate
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
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}


/* In exit_qemu(), we create a new I/O Port at 0xf4 (the iobase parameter).
0xf4 specifies on which port address hardware devices live. iosize specifies the
port size; here it's 4 bbytes for the isa-debug-exit device, which is why our
exit code (the value we write to the port) is in u32.
When a `value` is written to the I/O port specified by iobase, it causes QEMU to
exit with exit status `(value << 1) | 1`. We choose arbitrary values here. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,  // if all tests succeed
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


/* Since we removed the STD, we need to define the panic handler ourselves.
The panic handler is the function that the compiler calls when a panic occurs.
A panic is a Bad Thing (an unrecoverable error, like indexing out of bounds.)
Here, the PanicInfo parameter is an object that contains the file and line 
where the panic happened and an optional panic message. This panic() function 
should never return so it returns the "never" type !. Note it's not return
NOTHING, it's NEVER return.*/ 
#[panic_handler]  // this is an attribute
// the & means you've recieved a reference to PanicInfo through its address
// HOWEVER the & is NOT &mut, so you can't modify PanicInfo
fn panic(info: &PanicInfo) -> ! { 
    println!("{}", info);
    loop {}  // rip long comments #learning
}


/* What in the actual heck is &[&dyn Fn()]?
Fn(): Fn is a TRAIT. It represents "something that can be called like a function."
dyn: some obect implements Fn(), but we don't know what type it is.
&dyn Fn(): because trait objects are unsized, they must live behind a pointer.
So this expression basically means "a reference to something callable."
[&dyn Fn()] means you have an array of these where each element is &dyn Fn().
&[&dyn Fn()] is a reference to a slice of this array; we say "slice" but it can be
the full array. Rust internally stores a pointer to the first element AND arr len.
*/
#[cfg(test)]  // include only for testing purposes
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

/* Start of test cases! Tests are called from the _start entry point as test_main. */
#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
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