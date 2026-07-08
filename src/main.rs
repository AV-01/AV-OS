#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;

// std library requires an OS to work, which we don't have
// we don't have a runtime system, so we can't use main


// core library is still available and doesn't require OS support

// this one gives us info on errors that occur
use core::panic::PanicInfo;

// during fatal errors, call this function
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}


#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests{
        test();
    }

    exit_qemu(QemuExitCode::Success);
}


#[test_case]
fn trivial_assertion() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}

#[unsafe(no_mangle)] // force compiler to name function "_start"
pub extern "C" fn _start() -> ! {
    let name = "Aadya";
    println!("Hello World again from AV!");
    println!("Now one with arguments: {}", name);
    
    #[cfg(test)]
    test_main();

    loop{}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}