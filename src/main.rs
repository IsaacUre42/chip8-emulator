// Thanks to: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

use std::iter::zip;
use std::ops::Div;
use std::time::{Duration, Instant};
use minifb::{Key, Window, WindowOptions, ScaleMode};
use fastrand::u8;
use minifb::Key::{Key1, Key2, Key3, Key4};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const ON_COLOR: u32 = 0x00FF00;
const OFF_COLOR: u32 = 0;

struct Emulator {
    memory: [u8; 4096],
    program_counter: u16,
    index_register: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
    buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    prev_buffer: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    color_display_buffer: Vec<u32>,
    window: Window,
    shift_mode: bool,
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut window = Window::new(
            "Test - ESC to exit",
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            WindowOptions {
                scale_mode: ScaleMode::AspectRatioStretch,
                resize: true,
                ..WindowOptions::default()
            }).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        
        Emulator {
            memory: [0; 4096],
            program_counter: 512,
            index_register: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            prev_buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            color_display_buffer: vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            window,
            shift_mode: false
        }
    }
    
    pub fn update_display(&mut self) -> bool {
        if self.window.is_open() & !self.window.is_key_down(Key::Escape) {
            self.translate_buffer();
            self.window.update_with_buffer(&*self.color_display_buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT).unwrap();
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
            return true
        }
        false
    }

    pub fn update(&mut self) {
        //Fetch
        let instruction = (self.memory[self.program_counter as usize] as u16) << 8 | (self.memory[(self.program_counter + 1) as usize] as u16);
        self.program_counter += 2;

        //Decode
        let nibble = (instruction & 0xF000) >> 12;
        let x = ((instruction & 0x0F00) >> 8) as u8;
        let y = ((instruction & 0x00F0) >> 4) as u8;
        let n = (instruction & 0x000F) as u8;
        let nn = (instruction & 0xFF) as u8;
        let nnn = instruction & 0xFFF;

        //Execute
        println!("{:04x}, {}", instruction, self.program_counter);
        match nibble {
            0x0 => {
                match n {
                    0x0 => {
                        // Clear Screen Buffer
                        for byte in self.buffer.iter_mut() {
                            *byte = 0;
                        }
                    }
                    0xE => {
                        // Exit from subroutine
                        self.program_counter = self.stack.pop().unwrap();
                    },
                    _ => {}
                }
            },
            0x1 => {
                // Jump
                self.program_counter = nnn;
            },
            0x2 => {
                // Enter Subroutine
                self.stack.push(self.program_counter);
                self.program_counter = nnn;
            },
            0x3 => {
                // Conditional skip
                if self.registers[x as usize] == nn {
                    self.program_counter += 2;
                }
            },
            0x4 => {
                // Conditional skip
                if self.registers[x as usize] != nn {
                    self.program_counter += 2;
                }
            },
            0x5 => {
                // Conditional skip
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }
            0x6 => {
                // Set
                self.registers[x as usize] = nn;
            },
            0x7 => {
                // Add
                self.registers[x as usize] += nn;
            },
            0x8 => {
                match n {
                    0x0 => {
                        // Set
                        self.registers[x as usize] = self.registers[y as usize]
                    },
                    0x1 => {
                        // OR
                        self.registers[x as usize] |= self.registers[y as usize]
                    },
                    0x2 => {
                        // AND
                        self.registers[x as usize] &= self.registers[y as usize]
                    },
                    0x3 => {
                        // XOR
                        self.registers[x as usize] ^= self.registers[y as usize]
                    },
                    0x4 => {
                        // Add
                        let operation = self.registers[x as usize].overflowing_add(self.registers[y as usize]);
                        self.registers[x as usize] = operation.0;
                        self.registers[0xF] = if operation.1 {1} else {0};
                    },
                    0x5 => {
                        // Sub
                        let operation = self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                        self.registers[x as usize] = operation.0;
                        self.registers[0xF] = if operation.1 {0} else {1};
                    },
                    0x6 => {
                        // Shift
                        if self.shift_mode {
                            self.registers[x as usize] = self.registers[y as usize];
                        }
                        let bit = self.registers[x as usize] & 1;
                        self.registers[x as usize] >>= 1;
                        self.registers[0xF] = if bit == 0 {0} else {1};
                    },
                    0x7 => {
                        // Sub
                        let operation = self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                        self.registers[x as usize] = operation.0;
                        self.registers[0xF] = if operation.1 {0} else {1};
                    },
                    0xE => {
                        // Shift
                        if self.shift_mode {
                            self.registers[x as usize] = self.registers[y as usize];
                        }
                        let bit = self.registers[x as usize] & 0b1000_0000;
                        self.registers[x as usize] <<= 1;
                        self.registers[0xF] = if bit == 0 {0} else {1};
                    },
                    _ => {}
                }
            },
            0x9 => {
                // Conditional skip
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                }
            },
            0xA => {
                // Set Index Register I
                self.index_register = nnn;
            },
            0xB => {
                // Jump plus offset
                self.program_counter = nnn + (self.registers[0] as u16)
            },
            0xC => {
                let random = u8(0..u8::MAX);
                self.registers[x as usize] = nn & random;
            }
            0xD => {
                // Draw Stuff
                let start = self.index_register as usize;
                let end = (self.index_register + (n as u16)) as usize;
                let sprite: Vec<u8> = self.memory[start..end].to_vec();
                let x_pos = self.registers[x as usize] as u16% DISPLAY_WIDTH as u16;
                let y_pos = self.registers[y as usize] as u16 % DISPLAY_HEIGHT as u16;
                let position = x_pos + (y_pos * DISPLAY_WIDTH as u16);
                self.draw_sprite(&sprite, position as usize);
            }
            0xE => {
                // Skip if key
                match nn {
                    0x9E => {
                        if self.check_input(self.registers[x as usize]) {
                            self.program_counter += 2;
                        }
                    },
                    0xA1 => {
                        if !self.check_input(self.registers[x as usize]) {
                            self.program_counter += 2;
                        }
                    }
                    _ => {}
                }
            }
            0xF => {
                // Timers
                match nn {
                    0x07 => {
                        self.registers[x as usize] = self.delay_timer;
                    }
                    0x15 => {
                        self.delay_timer = self.registers[x as usize];
                    }
                    0x18 => {
                        self.sound_timer = self.registers[x as usize];
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn check_input(&self, key: u8) -> bool {
        match key {
            0x0 => {self.window.is_key_down(Key::X)},
            0x1 => {self.window.is_key_down(Key1)},
            0x2 => {self.window.is_key_down(Key2)},
            0x3 => {self.window.is_key_down(Key3)},
            0x4 => {self.window.is_key_down(Key::Q)},
            0x5 => {self.window.is_key_down(Key::W)},
            0x6 => {self.window.is_key_down(Key::E)},
            0x7 => {self.window.is_key_down(Key::A)},
            0x8 => {self.window.is_key_down(Key::S)},
            0x9 => {self.window.is_key_down(Key::D)},
            0xA => {self.window.is_key_down(Key::Z)},
            0xB => {self.window.is_key_down(Key::C)},
            0xC => {self.window.is_key_down(Key4)},
            0xD => {self.window.is_key_down(Key::R)},
            0xE => {self.window.is_key_down(Key::F)},
            0xF => {self.window.is_key_down(Key::V)},
            _ => {false}
        }
    }

    fn draw_sprite(&mut self, sprite: &Vec<u8>, position: usize) {
        let mut y = 0;
        for byte in sprite {
            let offset = (position + (y*DISPLAY_WIDTH)) % 8;
            let mut index = (position + (y*DISPLAY_WIDTH)).div(8);
            if offset == 0 {
                self.buffer[index] ^= byte;
            } else {
                let combined = ((self.buffer[index] as u16) << 8) | (self.buffer[index + 1] as u16);
                let flipper = (*byte as u16) << (8 - offset);
                let flipped = (combined ^ flipper) & ((0xFFu16) << (8 - offset));
                let first = (flipped >> 8) as u8;
                let second = flipped as u8;
                self.buffer[index] = first | ((self.buffer[index] >> (8-offset)) << (8 - offset));

                if (index + 1) % DISPLAY_WIDTH != 0 {
                    // Stop sprites from wrapping
                    // Assumes display width divisible by 8
                    self.buffer[index + 1] = second | (self.buffer[index + 1] << offset >> offset);
                }
            }

            y += 1;
        }
    }

    fn translate_buffer(&mut self) {
        // I had the algorithm... Claude had the code
        // Performs an XOR to see which pixels have changed
        for (byte_idx, (&current_byte, &new_byte)) in
            zip(self.prev_buffer.iter(), self.buffer.iter()).enumerate() {
            let changed_bits = current_byte ^ new_byte;

            if changed_bits != 0 {
                for bit_offset in 0..8 {
                    if (changed_bits & (1 << bit_offset)) != 0 {
                        let pixel_index = (byte_idx * 8) + (7-bit_offset);

                        if pixel_index < self.color_display_buffer.len() {
                            // Update color based on new pixel state
                            let new_bit = (new_byte & (1 << bit_offset)) != 0;
                            self.color_display_buffer[pixel_index] =
                                if new_bit { ON_COLOR } else { OFF_COLOR };
                        }
                    }
                }
            }
        }
        self.prev_buffer = self.buffer;
    }
    
    fn load_font(&mut self) {
        let font: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80]; // F

        self.memory[0x050..0x050 + font.len()].copy_from_slice(&font);
    }
}

fn main() {
    let mut emulator = Emulator::new();

    let mut running = true;
    // 700 Instructions per second standard
    let target_hz = Duration::from_secs_f64(1.0/700.0);
    // Limit to 60 fps
    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);

    load_rom_into_memory(&mut emulator.memory, "roms/ibm.ch8".to_string(), 512);
    

    let mut frame_delay = Instant::now();
    let mut hz_delay = Instant::now();
    while running {
        let frame_elapsed = frame_delay.elapsed();
        let hz_elapsed = hz_delay.elapsed();

        if hz_elapsed > target_hz {
            emulator.update();
            hz_delay = Instant::now();
        }

        if frame_elapsed > target_frame_time {
            running = emulator.update_display();
            frame_delay = Instant::now();
        }
    }
}

fn load_rom_into_memory(memory: &mut [u8; 4096], filepath: String, position: usize) {
    // https://www.reddit.com/r/rust/comments/dekpl5/comment/f2wminn/

    match std::fs::read(filepath) {
        Ok(bytes) => {
            memory[position..(position + bytes.len())].copy_from_slice(&*bytes);
        }
        Err(e) => {
            println!("Failed to load rom!");
            panic!("{}", e);
        }
    }
}

