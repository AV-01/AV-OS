mod vga_buffer;

// std library requires an OS to work, which we don't have
#![no_std]
// we don't have a runtime system, so we can't use main
#![no_main]

// core library is still available and doesn't require OS support

// this one gives us info on errors that occur
use core::panic::PanicInfo;

// during fatal errors, call this function
#[panic_handler] // function attribute
fn panic(_info: &PanicInfo) -> ! {
    loop{} // infinite loop. effectively stops us from running corrupted code
}

static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)] // force compiler to name function "_start"
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop{}
}


