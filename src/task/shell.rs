use crate::fs::FILESYSTEM;
use crate::interrupts::get_timer;
use crate::print;
use crate::println;
use crate::task::keyboard::ScancodeStream;
use crate::vga_buffer::{BUFFER_WIDTH, Color, WRITER};
use alloc::string::String;
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

static mut CHAOS_MODE: bool = false;
static mut CHAOS_COLOR: Color = Color::Blue;

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
                                        if CHAOS_MODE {
                                            CHAOS_COLOR = CHAOS_COLOR.next_color();
                                            WRITER.lock().set_colors(CHAOS_COLOR, Color::Black);
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
pub fn execute_command(input: &String) {
    let mut parts = input.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return,
    };

    WRITER.lock().set_colors(Color::LightBlue, Color::Black);

    match command {
        "help" => {
            println!("Available commands:\n");

            const HELP_RESULTS: [(&str, &str); 10] = [
                ("help", "shows this list"),
                ("clear", "clear the screen"),
                ("echo <param>", "repeats the given param"),
                ("uptime", "shows how many ticks ran since powered on"),
                ("ls", "lists the files stored"),
                ("write <filename> <content>", "stores <content> in a file named <filename>"),
                ("read <filename>", "reads a stored file"),
                ("delete <filename>", "deletes a given file"),
                ("chaos", "toggles chaos mode"),
                ("shutdown", "closes QEMU")
            ];

            for (key, desc) in HELP_RESULTS {
                let num_spaces = BUFFER_WIDTH - 2 - key.len() - desc.len();
                print!("  {}", key);
                for _ in 0..num_spaces {
                    print!(" ");
                }
                println!("{}", desc);
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

        "chaos" => unsafe {
            CHAOS_MODE = !CHAOS_MODE;
            println!(
                "Chaos mode toggled: {}",
                if CHAOS_MODE { "ON" } else { "OFF" }
            );
        },

        "echo" => {
            // serial_println!("{}", input);
            let echo = input.trim().splitn(2, " ").nth(1).unwrap_or("");
            println!("{}", echo);
        }

        "uptime" => {
            // use crate:interrupts::get_timer;
            println!("Ticks: {}", get_timer());
        }

        "ls" => {
            let fs = FILESYSTEM.lock();
            if fs.files.is_empty() {
                println!("No files currently!");
            }
            else {
                println!("Files found: \n");
                for filename in fs.files.keys() {
                    println!("{}", filename);
                }
            }
        }

        "write" => {
            let mut comms = input.trim().splitn(3, ' ');
            let _command = comms.next();
            let path = comms.next();
            let content = comms.next();

            match (path, content) {
                (Some(name), Some(text)) => {
                    let mut fs = FILESYSTEM.lock();
                    
                    let mut unique_name = true;

                    for filename in fs.files.keys() {
                        if name == filename {
                            unique_name = false;
                        }
                    }
                    if unique_name {
                        fs.write_file(
                            String::from(name),
                            text.as_bytes().to_vec(),
                        );
                        println!("File '{}' written successfully.", name);
                    }
                    else {
                        println!("{} already exists! Use a unique name.", name);
                    }
                }

                _ => {
                    println!("Usage: write <filename> <content>");
                }
            }
        }

        "read" => {
            let mut comms = input.trim().splitn(2, ' ');
            let _command = comms.next();
            let path = comms.next();

            match path {
                Some(path) => {
                    let mut fs = FILESYSTEM.lock();
                    let content = fs.read_file(path);

                    match content {
                        Some(data) => {
                            let result = core::str::from_utf8(data.as_slice());
                            match result {
                                Ok(result) => {
                                    println!("{}", result);
                                }

                                Err(_) => {
                                    println!("An error occured! File is not valid utf-8");
                                }
                            }
                        }
                        
                        None => {
                            println!("File not found. Use command 'ls' to check if it exists!");
                        }
                    }

                }
                None => {
                    println!("Usage: read <path>");
                    println!("Make sure to include a file name!")
                }
            }
        }

        "delete" => {
            let mut parts = input.trim().splitn(2, ' ');
            let _c = parts.next();
            let filename = parts.next();

            match filename {
                Some(filename) => {
                    let mut fs = FILESYSTEM.lock();
                    let result = fs.delete_file(filename);
                    if result {
                        println!("File {} deleted!", filename);
                    }
                    else {
                        println!("Error! File {} not found. Use command 'ls' to check if it exists!", filename);
                    }
                }

                None => {
                    println!("Usage: delete <filename>");
                    println!("Make sure to include a filename!");
                }
            }
        }

        command => {
            println!("Error: Command '{}' not found", command);
        }
    }

    WRITER.lock().set_colors(Color::Yellow, Color::Black);
}
