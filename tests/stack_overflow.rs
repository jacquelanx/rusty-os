#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]


use core::panic::PanicInfo;
use rusty_os::serial_print;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use rusty_os::{exit_qemu, QemuExitCode, serial_println};


/* We do a quick rewired double fault handler because here we are testing
the double fault so we WANT it to happen. */
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}


/* Reinitialize different IDT that holds the different double fault handler. */
lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rusty_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}


pub fn init_test_idt() {
    TEST_IDT.load();
}


#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    rusty_os::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}


#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
    volatile::Volatile::new(0).read(); // prevent tail recursion optimizations
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rusty_os::test_panic_handler(info)
}