#![cfg_attr(target_os = "none", no_std)]
#![no_main]

#[macro_use]
mod nadk;

#[macro_use]
mod text_editor;

use crate::nadk::display::Color565;
use crate::nadk::keyboard::{Key, InputManager};
use crate::nadk::utils::wait_ok_released;
use crate::nadk::time;

use crate::text_editor::editor::{Editor, ColorPalette};

// The app name must be a C string and the app name size must include the end line NULL character
configure_app!(b"Rust Code\0", 10, "../target/icon.nwi", 4250);

// Setup the heap allocator
setup_allocator!();

#[unsafe(no_mangle)]
fn main() {
    // You must call setup_allocator!() before
    init_heap!();
    wait_ok_released();
    
    // default palette
    let palette = ColorPalette { 
        background: Color565::from_rgb888(0x2d,0x35,0x3b),
        separator : Color565::from_rgb888(0x4f, 0x58, 0x5e),
        text_color: Color565::from_rgb888(0xc0, 0xb6, 0x9e),
    };

    let mut editor = Editor::from_content("file.rs", "\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nhello, I'm rust\nA slick modern language\nI am very good for low level tasks\nBut my borrow checker is\nsometimes annoying", palette);
     
    let mut input_manager = InputManager::new();
    
    let mut old_time = time::get_current_time_millis();
    let mut dt = 0;

    loop {
        input_manager.scan();
        if input_manager.is_keydown(Key::Home) {
            break;
        }
        
        editor.update(dt as u32);

        let new_time = time::get_current_time_millis();
        dt = new_time - old_time;
        old_time = new_time;
    }
}
