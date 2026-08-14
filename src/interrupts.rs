use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::gdt;
use lazy_static::lazy_static;


/* Creates a new Interrupt Descriptor Table. It's mutable because we later
register our handler functions on it ourselves. It's a lazy static because
we need the IDT to be a valid reference for the entirety of our program
since we need to reach for it every time there's an Exception. */
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}


extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}


extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}


pub fn init_idt() {
    IDT.load();
}


#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}