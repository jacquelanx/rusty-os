use uart_16550::{Config, Uart16550Tty, backend::PioBackend};
use spin::Mutex;
use lazy_static::lazy_static;


/* In the signature, ref means lazy_static stores a reference to a heap-allocated 
object. Let's break down the object type Mutex<Uart16550Tty<PioBackend>>: 
Uart16550Tty<PioBackend> is the UART. PioBackend means that the UART communicates
with I/O ports. We wrap it in Mutex<...> so that multiple pieces of code can share
the UART (this ensures only one Writer prints at one time, for example).
The UART object is then created by Uart16550Tty::new_port(0x3F8, Config::default()):
0x3F8 is the first I/O address for COM1 (UART has multiple ports bc it's complex). 
Config::default() creates the default UART settings.
*/
lazy_static! {
    pub static ref SERIAL1: Mutex<Uart16550Tty<PioBackend>> = Mutex::new(unsafe {
        Uart16550Tty::new_port(0x3F8, Config::default())
            .expect("failed to initialize UART")
    });
}


/* Recieves an argument of the type Arguments, passed in by serial_print.
The lock() function gives exclusive access to the UART. */
#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts; 

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}


/* Prints content from the kernel to the host through the serial interface.
$($arg:tt)* basically means "take all the arguments": * means "match zero or
more things" and tt means each thing is some chunck of Rust syntax. 
format_args() creates an object in the format Arguments {format_string: ...,
values: ...} */
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}


/* Prints to the host through the serial interface, appending a newline. 
There are 3 cases here: case 1 prints a new line for an empty input, case 2
takes an input $fmt and concatenates that with a new line, case 3 handles
format strings and appends a new line to the format string. NOTE that this
function calls serial_print.
*/
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}