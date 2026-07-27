// The whole idea in this module is that we CAN directly just do pointer stuff 
// to write to VGA at 0xb8000, but using structs makes it safer and more efficient.
// # i (dont rlly) love rust :D

use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

/* Allow dead_code disables warnings that a variant (eg. Pink) is unused. 
Derive Debug, Clone... lets you perform convenient operations with this Enum.
repr(u8) forces each Color to be exactly one byte (u8) because VGA expects
one byte exactly, so White, for example, is 00001111. Using a u4 would also
work since our max value is 15 but Rust doesn't have a u4 type.
*/
#[allow(dead_code)]  // an attribute
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {  // pub means other modules can use this enum
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}


/* This represents a FULL color code with foreground AND background.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]  // makes ColorCode have same memory layout as u8
struct ColorCode(u8);  // defines struct called ColorCode with u8 field

// Isn't it redundant to use a struct with just one field? Why not just use a
// u8 value everywhere? Using a struct enforces type safety for this special
// ColorCode type we've defined!

impl ColorCode {  // impl: here are the methods that belong to ColorCode
    // this is a constructor that creates a new color code
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


/* This represents a complete char (2 bytes) with BOTH color and character. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]  // gurantees exact field order
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}


/* This represents the text buffer. There are 80 cols x 25 rows = 2000 characters
where each one is a ScreenChar. It tells Rust to view memory starting 0xb8000 like:
Buffer {
    chars: [
        [...80 ScreenChars...],
        [...80 ScreenChars...],
        ...
        25 rows
    ]
} 
We also need to use the Volatile wrapper (which requires us to use the write() 
method) because even though the hardware reads memory at the VGA region starting
at 0xb8000, the program doesn't, so whatever is there might be optimized away.
Volatile says "don't do that." */
const BUFFER_HEIGHT: usize = 25;  // usize: an unsigned int with same memory size as machine
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


/* A bookkeeping tool that knows where you are (column keeps track of location)
and what color to use. The buffer field is a reference to the initialized Buffer;
mut means you can modify the Buffer object and 'static means that this pointer
is valid for the entire life of the program.
*/
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /* This function writes one ASCII byte onto the screen. &mut self means: 
    give me a mutable pointer to the Writer object I'm being called on. 
    The function signature can also be written like:
    pub fn write_byte(self: &mut Writer, byte: u8) but we use the shorthand.
    */
    pub fn write_byte(&mut self, byte: u8) {
        match byte {                   // this is a switch()
            b'\n' => self.new_line(),  // if the byte literal is '\n'

            byte => {              // if byte is anything else, write it
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }
}

impl Writer {
    /* Move every character one line up and start at the start of the last line. */
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
}

impl Writer {
    /* Clears a row by overwriting all of its characters with a space character. */
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl Writer {
    /* This function takes in a reference to a string as its 2nd argument.
    We then convert the string to bytes and print them one-by-one. 
    */
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),  // this is a ■ char LMAO
            }
        }
    }
}

impl fmt::Write for Writer {
    /* We want to use the core::fmt::Write trait so that we can use the write!
    macro to print ints, floats, etc. To use this trait, we need to write a
    write_str function. It does basically the same thing as our write_string
    function above so we just call that, basically.
    Returns either Ok(value) or Err(error). Here, returning () in Ok means
    returning nothing. */
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


/* Prints something to the screen. 
The awful-looking syntax for buffer:
1) Treat 0xb8000 as a RAW pointer to a mutable Buffer object.
2) However, the Writer struct wants a REFERENCE to a Buffer object, so we
dereference it to cast it as a mutable Buffer object.
3) We take a mutable reference to that object through &mut.
The address remains 0xb8000 throughout; we just changed its type from a raw
pointer to a refeence!

The explanation (fml :D): in Rust pointers and references are different...
Pointers can be valid or invalid; references are verified, valid addresses.
They're the same at runtime--both just addresses--but references are more
enforced and safe at compile time; they won't just segfault on you. 
*/
pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello! ");
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
}


/* Global Writer that can be used as an interface from other modules. It's a static,
which means that it's created at compile time and lasts for the entire duration of
the program. However, we need to make this lazy_static because Rust can't convert
raw pointers to references at compile time yet, which is what our buffer relies on.

Another problem: statics are immutable by default, but an immutable Writer is useless.
But a mutatable static is very dangerous because if you have 2 CPUs writing to the
Writer at the same time, it might induce a "data race" since the Writer is global,
which is a nasty bug. Our solution is to use the primitive spinlock, which ensures
that only one thread is active at a time. It DOES burn CPU though...

Mutex: Person A enters bathroom. OS puts Person B to sleep.
Spinlock: Person A enters bathroom. Person B stands outside and repeatedly asks
"Is it free?" "No." "Is it free?" "No." "Is it free?" "No." etc...
*/
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


/* Now we can use our global Writer to implement print and println macros. 
This makes our VGA usage very simple externally and we don't have to write
nasty code; we can just call println. 
*/
#[macro_export]  // this makes it available to everywhere in our crate
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}