use crate::print;
use crate::println;
use crate::task::keyboard::ScancodeStream;
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1, layouts, HandleControl};
use alloc::string::String;
use alloc::vec::Vec;

pub async fn run_shell() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    let mut input_buffer = String::new();

    print!("av_os> ");

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        match character {

                            '\n' => {
                                println!(); // Move to next line
                                execute_command(&input_buffer);
                                input_buffer.clear();
                                print!("av_os> ");
                            }
                            // On Backspace, remove last char and update screen
                            '\u{8}' => {
                                if !input_buffer.is_empty() {
                                    input_buffer.pop();
                                    crate::vga_buffer::backspace();
                                }
                            }
                            // Normal character entry
                            c => {
                                // Limit buffer size to prevent memory exhaustion
                                if input_buffer.len() < 64 {
                                    input_buffer.push(c);
                                    print!("{}", c);
                                }
                            }
                        }
                    }
                    DecodedKey::RawKey(_) => {}
                }
            }
        }
    }
}

pub fn execute_command(input: &str) {
    let mut parts = input.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return,
    };

    match command {
        "help" => {
            println!("Available commands:");
            println!("  help           - shows this list");
            println!("  clear          - clear the screen");
            println!("  shutdown       - closes machine");
        }

        "clear" => {
            crate::vga_buffer::clear_screen();
            // print!("av_os> ");
        }

        "shutdown" => {
            println!("Shutting down...");
            crate::exit_qemu(crate::QemuExitCode::Success);
            crate::hlt_loop();
        }

        command => {
            println!("Error: Command '{}' not found", command);
        }
    }
}