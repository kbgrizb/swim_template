#![no_std]

use num::cast::AsPrimitive;
use num::Integer;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    clear_screen, is_drawable, plot, plot_str, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH
};
use pluggable_interrupt_os::println;

use core::cmp::min;

const TASK_MANAGER_WIDTH: usize = 10;
const WIN_REGION_WIDTH: usize = BUFFER_WIDTH - TASK_MANAGER_WIDTH;
const MAX_OPEN: usize = 16;
const BLOCK_SIZE: usize = 256;
const NUM_BLOCKS: usize = 255;
const MAX_FILE_BLOCKS: usize = 64;
const MAX_FILE_BYTES: usize = MAX_FILE_BLOCKS * BLOCK_SIZE;
const MAX_FILES_STORED: usize = 30;
const MAX_FILENAME_BYTES: usize = 10;
const WIN_WIDTH: usize = (WIN_REGION_WIDTH - 3) / 2;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Document {
    window_size: (usize, usize),
    contents: [[char; 25]; 10], 
    num_letters_row: usize,
    next_letter: usize,
    cur_row: usize,
    cursor: usize,
    col: usize,
    row: usize,
}

pub struct SwimInterface {
    windows: [Document; 4],
    current_window: usize,
}

pub fn safe_add<const LIMIT: usize>(a: usize, b: usize) -> usize {
    (a + b).mod_floor(&LIMIT)
}

impl Default for SwimInterface {
    fn default() -> Self {
        Self {
            windows: [Document::default(); 4],
            current_window: 0,
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        Self {
            window_size: (WIN_WIDTH, 10),
            contents: [[' '; 25]; 10], 
            num_letters_row: 0,
            next_letter: 0,
            cur_row: 0,
            cursor: 0,
            col: 3, // Hardcoded
            row: 4, // Hardcoded
        }
    }
}

impl SwimInterface {
    fn letter_columns(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.windows[self.current_window].num_letters_row)
            .map(|n| safe_add::<BUFFER_WIDTH>(n, self.windows[self.current_window].col))
    }

    pub fn tick(&mut self) {
        self.clear_current();
        self.draw_current();
    }

    fn clear_current(&mut self) {
        for x in self.letter_columns() {
            plot(' ', x, self.windows[self.current_window].row, ColorCode::new(Color::Black, Color::Black));
        }
        let cursor_x = safe_add::<BUFFER_WIDTH>(self.windows[self.current_window].num_letters_row, self.windows[self.current_window].col);
        plot(' ', cursor_x, self.windows[self.current_window].row+self.windows[self.current_window].cur_row, ColorCode::new(Color::Black, Color::Black));
    }

    fn draw_current(&self) {
        for (index, window) in self.windows.iter().enumerate() {
            for row in 0..window.window_size.1 {
                for col in 0..window.window_size.0 {
                    plot(
                        window.contents[row][col],
                        col + window.col,
                        row + window.row,
                        ColorCode::new(Color::Cyan, Color::Black),
                );
            }
            }
        }
        let cursor_x = safe_add::<BUFFER_WIDTH>(self.windows[self.current_window].num_letters_row, self.windows[self.current_window].col);
        plot('|', cursor_x, self.windows[self.current_window].row+ self.windows[self.current_window].cur_row, ColorCode::new(Color::Cyan, Color::Cyan));

        //top left corner of each window, based on window size
        let window_corners = [
            (1, 2), 
            (self.windows[0].window_size.0, 2),
            (1, 2 + self.windows[0].window_size.1),
            (self.windows[0].window_size.0, 2 + self.windows[0].window_size.1)
            ];
        for x in 0..window_corners.len(){
        //plot horizontal lines
        for i in 0..self.windows[0].window_size.0{
            plot(
                '.',
                i + window_corners[x].0,
                window_corners[x].1,
                ColorCode::new(Color::White, Color::Black),
            );
            plot(
                '.',
                i + window_corners[x].0,
                window_corners[x].1 + self.windows[0].window_size.1,
                ColorCode::new(Color::White, Color::Black),
            )
        }
        //plot vertical lines
        for i in 0..self.windows[0].window_size.1{
            plot(
                '.',
                window_corners[x].0,
                window_corners[x].1 + i,
                ColorCode::new(Color::White, Color::Black),
            );
            plot(
                '.',
                window_corners[x].0 + self.windows[0].window_size.0 - 1,
                window_corners[x].1 + i,
                ColorCode::new(Color::White, Color::Black),
            )
        }
        for i in 0..self.windows[0].window_size.0{
            plot(
                '.',
                i + window_corners[self.current_window].0,
                window_corners[self.current_window].1,
                ColorCode::new(Color::Cyan, Color::Cyan),
            );
            plot(
                '.',
                i + window_corners[self.current_window].0,
                window_corners[self.current_window].1 + self.windows[self.current_window].window_size.1,
                ColorCode::new(Color::Cyan, Color::Cyan),
            )
        }
        for i in 0..self.windows[self.current_window].window_size.1{
            plot(
                '.',
                window_corners[self.current_window].0,
                window_corners[self.current_window].1 + i,
                ColorCode::new(Color::Cyan, Color::Cyan),
            );
            plot(
                '.',
                window_corners[self.current_window].0 + self.windows[self.current_window].window_size.0 - 1,
                window_corners[self.current_window].1 + i,
                ColorCode::new(Color::Cyan, Color::Cyan),
            )
        }
    }
    plot_str(
        "F1",
        window_corners[0].0 + self.windows[0].window_size.0 / 2,
        window_corners[0].1,
        ColorCode::new(Color::White, Color::Black),
    );
    plot_str(
        "F2",
        window_corners[1].0 + self.windows[1].window_size.0 / 2,
        window_corners[1].1,
        ColorCode::new(Color::White, Color::Black),
    );
    plot_str(
        "F3",
        window_corners[2].0 + self.windows[2].window_size.0 / 2,
        window_corners[2].1,
        ColorCode::new(Color::White, Color::Black),
    );
    plot_str(
        "F4",
        window_corners[3].0 + self.windows[3].window_size.0 / 2,
        window_corners[3].1,
        ColorCode::new(Color::White, Color::Black),
    );
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(c) => self.handle_raw(c),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::F1 => {
                self.current_window = 0;
        
            }
            KeyCode::F2 => {
                self.current_window = 1;
                if self.windows[self.current_window].num_letters_row == 0{
                    self.windows[self.current_window].col = 26;
                }
                
            }
            KeyCode::F3 => {
                self.current_window = 2;
                if self.windows[self.current_window].num_letters_row == 0{
                    self.windows[self.current_window].row = 13;
                }
            }
            KeyCode::F4 => {
                self.current_window = 3;
                if self.windows[self.current_window].num_letters_row == 0{
                    self.windows[self.current_window].col = 26;
                    self.windows[self.current_window].row = 13;
                }
                
            }
            _ => {}
        }
    }


    fn handle_unicode(&mut self, key: char) {
        if key == '\n'{
            self.modify_document(key);
        }
        if is_drawable(key) && self.windows[self.current_window].num_letters_row < BUFFER_WIDTH {
            
            self.clear_current(); 
            self.modify_document(key);
            self.draw_current();
        }
    }
    
    fn modify_document(&mut self, key: char) {
        let cur_doc = &mut self.windows[self.current_window];
    

        if key == '\n' || cur_doc.num_letters_row + 1 > cur_doc.window_size.0 - 4 {
            if cur_doc.cur_row < cur_doc.window_size.1 - 3{
            cur_doc.cur_row += 1;
            cur_doc.num_letters_row = 0;
            cur_doc.contents[cur_doc.cur_row][cur_doc.num_letters_row] = key;
            }
        } else {
            cur_doc.contents[cur_doc.cur_row][cur_doc.num_letters_row] = key;
            cur_doc.num_letters_row += 1;
        }
    }
    
}
