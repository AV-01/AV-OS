#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(av_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use av_os::task::{Task, executor::Executor};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

mod serial;
mod vga_buffer;
mod fs;

entry_point!(kernel_main);

extern crate alloc;
use crate::vga_buffer::{Color, WRITER};
use av_os::task::shell;
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use av_os::allocator;
    use av_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    WRITER.lock().set_colors(Color::LightRed, Color::Black);
    println!("WELCOME TO AV OS");
    println!("THE WORLD'S BEST OPERATING SYSTEM");
    println!("Use command 'help' to get started");
    WRITER.lock().set_colors(Color::LightRed, Color::Black);

    av_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // new
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    // executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(shell::run_shell())); // new
    executor.run();
}

// std library requires an OS to work, which we don't have
// we don't have a runtime system, so we can't use main

// core library is still available and doesn't require OS support

// this one gives us info on errors that occur

// during fatal errors, call this function
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    av_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    av_os::hlt_loop();
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    // new
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run(); // new
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
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
        let mut port = Port::<u32>::new(0xf4);
        port.write(exit_code as u32);

        let mut port_acpi = Port::<u16>::new(0x604);
        port_acpi.write(0x2000);

        let mut port_bochs = Port::<u16>::new(0xb004);
        port_bochs.write(0x2000);

        let mut port_vbox = Port::<u16>::new(0x4004);
        port_vbox.write(0x3400);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}
impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}
