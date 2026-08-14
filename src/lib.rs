/* Everything reusable (setup, shared functions) must live in lib.rs. Our
main.rs and tests are all BINARY crates, and in Rust a binary crate's content
can't be imported elsewhere, so everything we want to reuse must live in this
library crate (not binary crate). */

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]


pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;


/* Storytime... core contains the pieces of Rust that don't depend on an OS;
STD is built on core. Inside the core crate is a module called panic, which
contains a struct called PanicInfo. The use keyword is for ease of importing.
*/
use core::panic::PanicInfo;


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


/* A trait means "any type that implements me must provide these methods."
In this case, it's the run() method. run(&self) means the method borrows the
object. */
pub trait Testable {
    fn run(&self) -> ();
}


/* "For any type T, if T implements Fn(), then T also implements Testable."
Because T also implements Testable, it inherits run(). So we can just call
run() on various test functions and have repeated functionality across all
those functions. Note that Fn() is a function pointer or closure. */
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());  // print function name
        self();  // invoke test function; its type is &T where T: Fn()
        serial_println!("[ok]");
    }
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
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}


pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}


/* Normally, runtime procedures must happen before the main function is called
(eg. C runtime libraries, which invoke the entry point of the Rust runtime marked
by the _start item, which calls the main function). We don't have access to that
so we need to define our own entry point. The linker looks for a function called
`_start` by default.
*/
#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();  // call init to set up IDT for interrupts b4 running tests
    test_main();
    hlt_loop();
}


/* If we do loop{} in _start, this causes the CPU to spin endlessly, causing it
to consume nearly 100% of our CPU. What we want to do is to halt the CPU until the
next interrupt arrives. This lets the CPU enter a sleep state in between. */
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}


/* Since we removed the STD, we need to define the panic handler ourselves.
The panic handler is the function that the compiler calls when a panic occurs.
A panic is a Bad Thing (an unrecoverable error, like indexing out of bounds.)
Here, the PanicInfo parameter is an object that contains the file and line 
where the panic happened and an optional panic message. This panic() function 
should never return so it returns the "never" type !. Note it's not return
NOTHING, it's NEVER return. */ 
#[cfg(test)]
#[panic_handler]  // this is an attribute
// the & means you've recieved a reference to PanicInfo through its address
// HOWEVER the & is NOT &mut, so you can't modify PanicInfo
fn panic(info: &PanicInfo) -> ! { 
    test_panic_handler(info)
}


/* Loads interrupt initialization routine from the interrupts.rs module,
as well as GDT initialization. */
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    // enable external interrupts so the PICs can talk to the CPU
    x86_64::instructions::interrupts::enable();
}