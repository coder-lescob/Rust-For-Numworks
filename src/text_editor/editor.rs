// import from alloc
calc_use!(alloc::vec::Vec);
calc_use!(alloc::string::String);
calc_use!(alloc::format);
calc_use!(alloc::borrow::ToOwned);

// use the display module from nadk wrapper
use crate::nadk::display::{
    Color565, 
    SCREEN_RECT, 
    ScreenPoint, 
    ScreenRect, 
    draw_string, 
    push_rect_uniform
};
use crate::nadk::keyboard::Key;
use crate::text_editor::keyboard::{Keyboard, Modifiers};


/// a cursor with (x,y)
pub struct Cursor(pub u32, pub u32);

/// a color palette, for customization
#[derive(Copy, Clone)]
pub struct ColorPalette {
    pub background: Color565,
    pub separator : Color565,
    pub text_color: Color565,
}

/// represent the Editor current state
pub struct Editor {
    file_path: String,
    text: Vec<String>,
    cursor: Cursor,
    palette: ColorPalette,
    keyboard: Keyboard,
    modifiers: Modifiers,
}

impl Editor {

    /// create a new Editor from a content, easy to create new files like so
    pub fn from_content(file: &str, content: &str, palette: ColorPalette) -> Self {
        Self { 
            file_path: file.to_owned(), 
            text: content
                .split('\n')
                .map(|x| x.to_owned())
                .collect::<Vec<String>>(), 
            cursor: Cursor(0, 0),
            palette,
            keyboard: Keyboard::new(),
            modifiers: Modifiers { shift_active: false, alpha_active: false }
        }
    }

    pub fn update(&mut self, ms_dt: u32) {
        self.keyboard.scan(ms_dt);
        self.update_cursor();
        self.write_enabled();

        if self.keyboard.any_keydown() {
            self.redraw();
        }
    }
    
    // let the user write to the current editor
    fn write_enabled(&mut self) {

        // loop through all the keys
        for key in enum_iterator::all::<Key>() {
            if !self.keyboard.is_keydown(key) { continue; }

            // key down!
            if let Some(ch) = Key::get_matching_char(&key, self.modifiers.shift_active, self.modifiers.alpha_active) {
                // write the character to the buffer
                self.write_char_to_text(ch);
                continue;
            }

            match key {
                Key::Shift => self.modifiers.shift_active = !self.modifiers.shift_active,
                Key::Alpha => self.modifiers.alpha_active = !self.modifiers.alpha_active,
                Key::Exe   => self.insert_new_line(),
                Key::Backspace => self.delete_char_before_cursor(),
                _ => ()
            }
        }
    }
    
    /// writes a character to the text of the editor
    fn write_char_to_text(&mut self, ch: char) {

        // write to the text
        if let Some((byte_idx, _)) = self.text[self.cursor.1 as usize]
                                        .char_indices()
                                        .nth(self.cursor.0 as usize) {
            self.text[self.cursor.1 as usize].insert(self.cursor.0 as usize, ch);
        }
        else {
            self.text[self.cursor.1 as usize].push(ch);
        }

        // advence the cursor
        self.cursor.0 += 1;
    }
    
    /// deletes the character before the cursor and moves the cursor accordingly
    fn delete_char_before_cursor(&mut self) {
        // put the character to the end of the line if further
        self.cursor.0  = self.cursor.0.min(self.text[self.cursor.1 as usize].len() as u32);

        if self.cursor.0 == 0 {
            if self.cursor.1 == 0 { return; }

            // borrow line content
            let current_line = String::from(&self.text[self.cursor.1 as usize]);

            // go to previous line and concat the two lines
            self.cursor.1 -= 1;
            self.cursor.0 = self.text[self.cursor.1 as usize].len() as u32;
            self.text[self.cursor.1 as usize].push_str(&current_line);

            // remove the current line
            self.text.remove(self.cursor.1 as usize + 1);

            return;
        }

        self.cursor.0 -= 1;
        self.text[self.cursor.1 as usize].remove(self.cursor.0 as usize);
    }
    
    /// insert a new line
    fn insert_new_line(&mut self) {
        // get the text before and after the cursor
        self.cursor.0  = self.cursor.0.min(self.text[self.cursor.1 as usize].len() as u32);
        let before = String::from(&self.text[self.cursor.1 as usize][..self.cursor.0 as usize]);
        let after  = if self.cursor.0 < self.text[self.cursor.1 as usize].len() as u32 {
            String::from(&self.text[self.cursor.1 as usize][self.cursor.0 as usize..])
        }
        else { String::new() };

        // deletes some of the content of the current line
        self.text[self.cursor.1 as usize] = before;

        // creates a new line after the current
        self.cursor.1 += 1;
        self.cursor.0 = 0;
        self.text.insert(self.cursor.1 as usize, after);
    }
    
    /// moves the cursor around
    fn update_cursor(&mut self) { 
        if self.keyboard.is_keydown(Key::Down) {
            if self.cursor.1 + 1 < self.text.len() as u32 {
                self.cursor.1 += 1;
            }
        }

        if self.keyboard.is_keydown(Key::Up) {
            if self.cursor.1 as i32 - 1 >= 0i32 {
                self.cursor.1 -= 1;
            }
        }

        if self.keyboard.is_keydown(Key::Left) {
            if self.cursor.0 as i32 - 1 >= 0i32 {
                self.cursor.0 = self.cursor.0.min(self.text[self.cursor.1 as usize].len() as u32);
                self.cursor.0 -= 1;
            }
        }

        if self.keyboard.is_keydown(Key::Right) {
            if self.cursor.0 + 1 <= self.text[self.cursor.1 as usize].len() as u32 {
                self.cursor.0 += 1;
            }
        }
    }
    
    /// redraws the editor to the screen
    pub fn redraw(&self) {
        // redraw background
        push_rect_uniform(SCREEN_RECT, self.palette.background);

        // draw the separator
        push_rect_uniform(ScreenRect::new(17, 0, 1, SCREEN_RECT.height), self.palette.separator);
    
        // we ideally want the cursor to always be on line 8 of the screen, 
        // except on the border of file
        let first_line = (self.cursor.1 as i32 - 8).min(self.text.len() as i32 - 16).max(0);

        // render all the lines and line numbers
        for i in 0..(self.text.len() - first_line as usize).min(16) {
            let line = first_line as usize + i;

            // render line number
            draw_string(
                &format!("{:02}", (line + 1) % 100), 
                ScreenPoint::new(1, 15 * i as u16), 
                false, self.palette.separator, self.palette.background
            );
        
            // render line
            draw_string(
                &self.text[line], 
                ScreenPoint::new(20, 15 * i as u16), 
                false, self.palette.text_color, self.palette.background
            );
        }

        // draw the cursor
        let cursor_line   = self.cursor.1 as i32 - first_line;
        let cursor_column = self.cursor.0.min(self.text[self.cursor.1 as usize].len() as u32) as u16;
        push_rect_uniform(ScreenRect::new(7 * cursor_column + 20, 15 * cursor_line as u16, 1, 15), Color565::from_rgb888(255, 255, 255));
    }
}
