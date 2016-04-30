#![feature(const_fn)]
#![no_std]


extern crate spin;


use spin::Mutex;
use core::fmt;


pub const DEFAULT_COLOR: ColorCode = ColorCode::new(Color::LightGreen, Color::Black);
pub const COLS: isize = 80;
pub const ROWS: isize = 25;


#[repr(u8)]
pub enum Color {
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
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Copy, Clone)]
pub struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct VgaCell {
    character: u8,
    color: ColorCode,
}

pub static BUFFER: Mutex<VgaBuffer> = Mutex::new(VgaBuffer {
    buffer: [
        VgaCell {
            character: ' ' as u8,
            color: DEFAULT_COLOR,
        }; (ROWS * COLS * 2) as usize
    ],
    position: 0,
});

pub struct VgaBuffer {
    buffer: [VgaCell; (ROWS * COLS * 2) as usize],
    position: usize,
}

impl VgaBuffer {
    fn write_byte(&mut self, byte: u8, color: ColorCode) {
        if byte == ('\n' as u8) {
            let current_line = (self.position as isize) / COLS;

            let next_line = if current_line + 1 > ROWS {

                let end = ROWS * COLS;

                for i in COLS..(end) {
                    let prev = i - COLS;
                    self.buffer[prev as usize] = self.buffer[i as usize];

                }

                for i in (end - COLS)..(end) {
                    let cell = &mut self.buffer[i as usize];
                    *cell = VgaCell {
                        character: ' ' as u8,
                        color: DEFAULT_COLOR,
                    };
                }

                ROWS - 1
            } else {
                current_line + 1
            };

            self.position = (next_line * COLS) as usize;
        } else {
            let cell = &mut self.buffer[self.position];

            *cell = VgaCell {
                character: byte,
                color: color,
            };

            self.position += 1;
        }
    }

    fn reset_position(&mut self) {
        self.position = 0;
    }

    pub fn flush(&self) {
        unsafe {
            let vga = 0xb8000 as *mut u8;
            let length = self.buffer.len() * 2;
            let buffer: *const u8 = core::mem::transmute(&self.buffer);
            core::intrinsics::copy_nonoverlapping(buffer, vga, length);
        }
    }

    fn clear(&mut self) {
        for i in 0..(ROWS * COLS * 2) {
            let cell = &mut self.buffer[i as usize];
            *cell = VgaCell {
                character: ' ' as u8,
                color: DEFAULT_COLOR,
            };
        }

        self.reset_position();
        self.flush();
    }
}

impl fmt::Write for VgaBuffer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        let color = DEFAULT_COLOR;
        for byte in s.bytes() {
            self.write_byte(byte, color)
        }
        Ok(())
    }
}

pub fn clear() {
    let mut b = BUFFER.lock();
    b.clear();
}

#[macro_export]
macro_rules! vga_println {
    ($fmt:expr) => (vga_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (vga_print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let mut b = $crate::BUFFER.lock();
        b.write_fmt(format_args!($($arg)*)).unwrap();
        b.flush();
    });
}
