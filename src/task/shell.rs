use crate::print;
use crate::println;
use crate::task::keyboard::ScancodeStream;
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1, layouts, HandleControl};
use alloc::string::String;
use alloc::vec::Vec;
use crate::vga_buffer::{BUFFER_WIDTH, WRITER, Color};

static mut chaos_mode: bool = false;
static mut chaos_color: Color = Color::Blue; 

pub async fn run_shell() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    let mut input_buffer = String::new();

    print!("av_os> ");

    WRITER.lock().set_colors(Color::LightCyan, Color::Black);

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
                                WRITER.lock().set_colors(Color::LightCyan, Color::Black);
                            }
                            // On Backspace, remove last char and update screen
                            '\u{8}' => {
                                if !input_buffer.is_empty() {
                                    input_buffer.pop();
                                    crate::vga_buffer::backspace();
                                }
                            }
                            // Normal character entry
                            #[allow(static_mut_refs)]
                            c => {
                                // Limit buffer size to prevent memory exhaustion
                                if input_buffer.len() < 64 {
                                    input_buffer.push(c);
                                    unsafe {
                                        if chaos_mode {
                                            chaos_color = chaos_color.next_color();
                                            WRITER.lock().set_colors(chaos_color, Color::Black);
                                        }
                                    }
                                    print!("{}", c);
                                    // Restore normal input color after printing
                                    WRITER.lock().set_colors(Color::LightCyan, Color::Black);
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

#[allow(static_mut_refs)]
pub fn execute_command(input: &str) {
    let mut parts = input.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return,
    };

    WRITER.lock().set_colors(Color::LightBlue, Color::Black);

    match command {
        "help" => {
            println!("Available commands:");
            let avail_commands = ["help", "clear", "chaos", "shutdown"];
            let desc = ["shows this list", "clear the screen", "toggles chaos mode", "closes QEMU"];
            
            println!();

            for i in 0..avail_commands.len() {
                let num_spaces = BUFFER_WIDTH - 2 - avail_commands[i].len() - desc[i].len();
                print!("  {}", avail_commands[i]);
                for _ in 0..num_spaces {
                    print!(" ");
                }
                println!("{}", desc[i]);
            }
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

        "chaos" => {
            unsafe {
                chaos_mode = !chaos_mode;
                println!("Chaos mode toggled: {}", if chaos_mode { "ON" } else { "OFF" });
            }
        }

        command => {
            println!("Error: Command '{}' not found", command);
        }
    }

    WRITER.lock().set_colors(Color::Yellow, Color::Black);
}