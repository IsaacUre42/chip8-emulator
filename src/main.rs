// Thanks to: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

use std::iter::zip;
use std::ops::Div;
use std::thread;
use std::time::{Duration, Instant};
use minifb::{Key, Window, WindowOptions, ScaleMode};

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
    window: Window
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
            program_counter: 0,
            index_register: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            prev_buffer: [0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            color_display_buffer: vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            window
        }
    }
    
    pub fn update(&mut self) -> bool {

        if self.window.is_open() & !self.window.is_key_down(Key::Escape) {
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

    fn draw_sprite(&mut self, sprite: &Vec<u8>, position: u32) {
        let mut y = 0;
        for byte in sprite {
            let offset = (position as usize + (y*DISPLAY_WIDTH)) % 8;
            let mut index = (position as usize + (y*DISPLAY_WIDTH)).div(8);
            if offset != 0 {
                let mut untouched = self.buffer[index] >> (8 - offset) << (8 - offset);
                let mut touched = (self.buffer[index] ^ (byte >> offset)) << offset >> offset;
                self.buffer[index] = untouched | touched;
                index += 1;
                untouched = self.buffer[index] << offset >> offset;
                touched = (self.buffer[index] ^ byte << (8 - offset)) >> (8 - offset) << (8 - offset);
                self.buffer[index] = untouched | touched;
            } else {
                let reversed = byte.reverse_bits();
                self.buffer[index] ^= byte;
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
                        let pixel_index = (byte_idx * 8) + bit_offset;

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

    let sprite: Vec<u8> = vec![0xF0, 0x80, 0xF0, 0x80, 0x80];
    emulator.draw_sprite(&sprite, 0);
    emulator.draw_sprite(&sprite, 7);
    emulator.translate_buffer();

    // Limit to 60 fps
    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);

    while running {
        let frame_start = Instant::now();
        running = emulator.update();
        let frame_duration = frame_start.elapsed();
        if frame_duration < target_frame_time {
            thread::sleep(target_frame_time - frame_duration);
        }
    }
}

