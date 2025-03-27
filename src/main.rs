// Thanks to: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

use minifb::{Key, Window, WindowOptions, ScaleMode};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

fn main() {
    let mut display_buffer: Vec<u32> = vec![0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        DISPLAY_WIDTH,
        DISPLAY_HEIGHT,
        // WindowOptions::default())
        // .unwrap_or_else(|e| {
        //     panic!("{}", e);
        WindowOptions {
            scale_mode: ScaleMode::AspectRatioStretch,
            resize: true,
            ..WindowOptions::default()
        }).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);
    let mut memory: [u8; 4096] = [0; 4096];
    store_font(&mut memory);
    
    let mut counter = 0;
    for i in display_buffer.iter_mut() {
        *i = if memory[counter] == 1 {0xFFFFFF} else {0};
        counter += 1;
    }
    
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update the display buffer
        window.update_with_buffer(&display_buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT).unwrap();
    }
}

fn draw_sprite(sprite: Vec<u8>, buffer: &mut Vec<u32>, position: u32) {
    
}

fn setup() {
    let mut memory: [u8; 4096] = [0; 4096];
    let mut program_counter: u16 = 0x0;
    let mut index_register: u16 = 0x0;
    let mut stack: Vec<u16> = Vec::new();
    let mut delay_timer: u8 = 0;
    let mut sound_timer: u8 = 0;
    let mut registers: [u8; 16] = [0; 16];
}

fn store_font(memory: &mut [u8; 4096]) {
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
    
    memory[0x050..0x050 + font.len()].copy_from_slice(&font);
}

