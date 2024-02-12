mod mode;
mod text;
mod command;
mod task;

use std::{env::args, fs, io::{stdout, stdin, Write, BufWriter}};
use command::{Action, ActionStack, CmdAction};
use task::Task;
use termion::{color, style, raw::IntoRawMode, input::{TermRead, MouseTerminal}, event::Key, screen::AlternateScreen};
use text::Text;
use crate::mode::Mode;

#[derive(Clone, Copy)]
pub struct Coordinates {
    pub x: usize,
    pub y: usize
}

#[allow(dead_code)]
enum CursorStyle {
    Bar,
    Block,
    Underline
}

struct Size(u16, u16);

struct TextEditor {
    text: Text,
    cur_pos: Coordinates,
    cur_line: usize,
    view: TextView,
    terminal_size: Size,
    file_name: String,
    out: Box<dyn Write>,
    mode: Mode,
    task: Task,
    action_stack: ActionStack,
    revoking_action: bool,
}

struct TextView {
    lower_line: usize,
    upper_line: usize,
}

impl TextView {
    pub fn move_down(&mut self, n: usize) {
        self.upper_line += n;
        self.lower_line += n;
    }
    pub fn move_up(&mut self, n: usize) {
        if self.upper_line == 0 {
            return
        }
        if n > self.lower_line {
            self.upper_line -= self.lower_line;
            self.lower_line = 0;
        } else {
            self.upper_line -= n;
            self.lower_line -= n;
        }
    }
    pub fn upper_line(&self) -> usize{
        self.upper_line
    }
    pub fn lower_line(&self) -> usize{
        self.lower_line
    }
    pub fn shrink_upper(&mut self) {
        if self.upper_line > 0 {
            self.upper_line -= 1;
        }
    }
    pub fn expand_upper(&mut self) {
        self.upper_line += 1;
    }
}


impl TextEditor {
    pub fn new(file_name: &str) -> Self {
        let mut text = Text::new();
        let file_handle = fs::read_to_string(file_name).unwrap();
        for line in file_handle.lines() {
            text.push_line(line.to_string());
        }
        let text_length = file_handle.lines().count();
        let size = termion::terminal_size().unwrap();
        let view = TextView{lower_line: 0, upper_line: text_length.min(size.1 as usize - 1)};
        let mut out = MouseTerminal::from(AlternateScreen::from(BufWriter::with_capacity(
                1 << 14,
                stdout(),
            )))
            .into_raw_mode()
            .unwrap();
        write!(out, "{}", termion::cursor::Show).unwrap();
        let out = Box::new(out);
        TextEditor {
            text,
            cur_pos: Coordinates{x:1,y:1},
            cur_line: 1,
            view,
            terminal_size: Size(size.0, size.1),
            file_name: file_name.into(),
            out,
            mode: Mode::Normal,
            task: Task::default(),
            action_stack: ActionStack::default(),
            revoking_action: false
        }
    }

    #[cfg(test)]
    pub fn new_from_vec(lines: &Vec<String>) -> Self {
        let mut text = Text::new();
        for line in lines {
            text.push_line(line.clone());
        }
        let text_length = lines.len();
        let size = termion::terminal_size().unwrap();
        let view = TextView{lower_line: 0, upper_line: text_length.min(size.1 as usize - 1)};
        let mut out = BufWriter::with_capacity(
                1 << 14,
                vec![],
            );
        write!(out, "{}", termion::cursor::Show).unwrap();
        let out = Box::new(out);
        TextEditor {
            text,
            cur_pos: Coordinates{x:1,y:1},
            cur_line: 1,
            view,
            terminal_size: Size(size.0, size.1),
            file_name: "test_file".into(),
            out,
            mode: Mode::Normal,
            task: Task::default(),
            action_stack: ActionStack::default(),
            revoking_action: false
        }

    }

    fn flush(&mut self) {
        let pos = &self.cur_pos;
        let (mut old_x, old_y) = (pos.x, pos.y);

        self.print_text();
        self.show_bar();

        // FIXME: when '$' status is on, we should also move to the end of the line
        //          no matter what old_x is.
        old_x = old_x.min(self.len_of_cur_line());
        self.set_pos(old_x, old_y);
    }

    fn print_text(&mut self) {
        write!(self.out, "{}{}", termion::clear::All, termion::cursor::Goto(1,1)).unwrap();
        for line in self.view.lower_line()..self.view.upper_line() {
            writeln!(self.out, "{}\r", self.text.line_at(line as usize)).unwrap();
        }
    }

    fn max_y(&self) -> u16 {
        self.terminal_size.1 - 1
    }

    fn show_bar(&mut self) {
        write!(self.out, "{}",termion::cursor::Goto(0, (self.terminal_size.1) as u16)).unwrap();
        write!(self.out, "{}{} line-count={} filename: {}, size: ({}, {}) line[{}-{}] pos[{}:{}] mode:{}{}",
                    color::Fg(color::Blue),
                    style::Bold,
                    self.text_length(),
                    self.file_name,
                    self.terminal_size.0,
                    self.terminal_size.1,
                    self.view.lower_line(),
                    self.view.upper_line(),
                    self.cur_pos.x,
                    self.cur_pos.y,
                    self.mode,
                    style::Reset
                ).unwrap();
    }

    fn set_pos(&mut self, x: usize, y: usize) {
        self.cur_pos.x = x;
        self.cur_pos.y = y;
        self.update_pos();
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        match style {
            CursorStyle::Bar => write!(self.out, "{}", termion::cursor::BlinkingBar),
            CursorStyle::Block => write!(self.out, "{}", termion::cursor::BlinkingBlock),
            CursorStyle::Underline => write!(self.out, "{}", termion::cursor::BlinkingUnderline),
        }.unwrap();
    }

    fn update_pos(&mut self) {
        write!(self.out, "{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)).unwrap();
    }

    pub fn revoke_action(&mut self, action: Option<CmdAction>) {
        self.revoking_action = true;

        if let Some(action) = action {
            let pos = action.pos;
            let cur_line = action.cur_line;
            self.set_pos(pos.x, pos.y);
            self.cur_line = cur_line;
            match action.action {
                Action::Delete => {

                },
                Action::Insert => {
                    action.contents.iter().for_each(|&a| {
                        if a == Key::Char('\t') {
                            for _ in 0..4 {
                                self.delete_cur_char();
                            }
                        } else {
                            self.delete_cur_char();
                        }
                    })
                }
            }
        }

        self.revoking_action = false;
    }

    fn len_of_cur_line(&self) -> usize {
        assert!(self.cur_line != 0);
        if self.mode == Mode::Normal {
            1.max(self.text.len_of_line_at(self.cur_line - 1))
        } else if self.mode == Mode::Insert {
            1.max(self.text.len_of_line_at(self.cur_line - 1) + 1)
        } else {
            unimplemented!()
        }
    }

    fn text_length(&self) -> usize {
        self.text.len()
    }

    fn cursor_at_end_of_line(&mut self) -> bool {
        self.cur_pos.x == self.len_of_cur_line()
    }
    fn move_to_end_of_line(&mut self) {
        self.cur_pos.x = self.len_of_cur_line();
    }
    fn move_to_start_of_line(&mut self) {
        self.cur_pos.x = 1;
    }
    fn move_to_first_char_of_line(&mut self) {
        self.cur_pos.x = 1;
        while self.cur_pos.x < self.len_of_cur_line() {
            if Self::is_blank(self.cur_char()) {
                self.cur_pos.x += 1;
            } else {
                return
            }
        }
    }
    fn inc_x(&mut self) {
        if self.cur_pos.x < self.len_of_cur_line() {
            self.cur_pos.x += 1;
        }
    }
    fn dec_x(&mut self) {
        if self.cur_pos.x > 1 {
            self.cur_pos.x -= 1;
        }
    }
    fn inc_y(&mut self) {
        if self.cur_pos.y < self.max_y().min(self.text_length() as u16).into() {
            self.cur_pos.y += 1;
        } else {
            if self.view.upper_line() < self.text_length() {
                self.view.move_down(1);
            }
        }
        if self.cur_line < self.text_length() {
            self.cur_line += 1;
        }
    }
    fn dec_y(&mut self) {
        if self.cur_pos.y > 1 {
            self.cur_pos.y -= 1;
        } else {
            self.view.move_up(1);

        }
        if self.cur_line > 1 {
            self.cur_line -= 1;
        }
    }
    fn forward_to_end_of_cur_word(&mut self) {
        assert!(Self::is_alphabet(self.cur_char()));
        while Self::is_alphabet(self.cur_char()) {
            let old_line = self.cur_line;
            if !self.forward_to_next_char() {
                return;
            }
            if self.cur_line != old_line {
                break;
            }
        }
        self.backward_to_next_char();
    }
    fn forward_to_start_of_cur_word(&mut self) {
        assert!(Self::is_alphabet(self.cur_char()));
        while Self::is_alphabet(self.cur_char()) {
            let old_line = self.cur_line;
            if !self.backward_to_next_char() {
                return;
            }
            if  self.cur_line != old_line {
                break;
            }
        }
        self.forward_to_next_char();
    }
    fn backward_to_start_of_next_word(&mut self) {
        self.backward_to_next_char();
        if Self::is_alphabet(self.cur_char()) {
            self.forward_to_start_of_cur_word();
        } else {
            // we are currently in blank char, need to find the next word
            while !Self::is_alphabet(self.cur_char()) {
                self.backward_to_next_char();
            }
        }
        self.forward_to_start_of_cur_word();
    }
    fn forward_to_end_of_next_word(&mut self) {
        self.forward_to_next_char();
        if Self::is_alphabet(self.cur_char()) {
            self.forward_to_end_of_cur_word();
        } else {
            // we are currently at non-alphabetic char, need to
            //      find the next alphabetic char
            while !Self::is_alphabet(self.cur_char()) {
                self.forward_to_next_char();
            }
        }
        self.forward_to_end_of_cur_word();
    }
    fn forward_to_start_of_next_word(&mut self) {
        while Self::is_alphabet(self.cur_char()) {
            let old_line = self.cur_line;
            if !self.forward_to_next_char() {
                return;
            }
            if self.cur_line != old_line {
                break;
            }
        }
        // we are currently in blank char, need to find the next word
        while !Self::is_alphabet(self.cur_char()) {
            self.forward_to_next_char();
        }
    }
    fn backward_to_next_char(&mut self) -> bool {
        if self.cur_pos.x == 1 {
            if self.cur_line > 1 {
                // move to the start of next line
                if self.cur_pos.y > 1 {
                    self.cur_pos.y -= 1;
                } else {
                    self.view.move_up(1);
                }
                self.cur_line -= 1;
                self.cur_pos.x = self.len_of_cur_line();
                return true;
            } else {
                // we hit the beginning of the file, just do nothing
                return false;
            }
        } else {
            self.cur_pos.x -= 1;
            return true;
        }
    }
    fn forward_to_next_char(&mut self) -> bool {
        if self.cur_pos.x == self.len_of_cur_line(){
            if self.cur_line < self.text_length()  {
                // move to the start of next line
                if self.cur_pos.y < self.max_y().into() {
                    self.cur_pos.y += 1;
                } else {
                    if self.view.upper_line() < self.text_length() {
                        self.view.move_down(1);
                    }
                }
                self.cur_line += 1;
                self.cur_pos.x = 1;
                return true;
            } else {
                // we hit the end of the file, just do nothing
                return false;
            }
        } else {
            self.cur_pos.x += 1;
            return true;
        }
    }
    fn new_line_ahead(&mut self) {
        self.text.add_line_before(self.cur_pos.y - 1, "".to_string());
        self.move_to_start_of_line();
        if self.text_length() < self.terminal_size.1 as usize - 1 {
            self.view.expand_upper();
        }
    }
    fn new_line_behind(&mut self) {
        self.text.new_line_at(self.cur_pos.y - 1, self.len_of_cur_line());
        self.inc_y();
        self.move_to_start_of_line();
        if self.text_length() < self.terminal_size.1 as usize - 1 {
            self.view.expand_upper();
        }
    }
    fn new_line(&mut self) {
        self.text.new_line_at(self.cur_pos.y - 1, self.cur_pos.x - 1);
        self.inc_y();
        self.move_to_start_of_line();
        if self.text_length() < self.terminal_size.1 as usize - 1 {
            self.view.expand_upper();
        }
    }
    fn cur_char(&mut self) -> char {
        self.text.char_at(self.cur_line - 1, self.cur_pos.x - 1)
    }
    fn is_alphabet(c: char) -> bool {
        c.is_alphanumeric()
    }
    fn is_blank(c: char) -> bool {
        c == ' ' || c == '\n' || c == '\t'
    }
    pub fn change_mode_immediately(&mut self, mode: Mode) {
        self.mode = mode;
    }
    pub fn delete_line_at(&mut self, index: usize) -> String {
        let res = self.text.delete_line_at(index);
        if self.text_length() < self.terminal_size.1 as usize - 1 {
            self.view.shrink_upper();
        }
        res
    }
    pub fn delete_cur_line(&mut self) -> String {
        let res = self.text.delete_line_at(self.cur_line - 1);
        if self.text_length() < self.terminal_size.1 as usize - 1 {
            self.view.shrink_upper();
        }
        res
    }

    pub fn delete_cur_char(&mut self) {

        if self.cur_char() == 0 as char {
            if self.cur_line < self.text_length() {
                let contents = self.delete_line_at(self.cur_line);
                self.text.append_str_at(self.cur_line - 1, self.len_of_cur_line(), contents);
            }
        } else {
            self.text.delete_at(self.cur_line - 1, self.cur_pos.x);
        }

    }
    fn run(&mut self) {
        self.flush();
        self.out.flush().unwrap();
        let stdin = stdin();
        for c in stdin.keys() {
            self.mode = self.mode.clone().handle(self, c.unwrap());
            if self.mode == Mode::Exit {
                break;
            }
            self.flush();
            self.out.flush().unwrap();
        }
    }
}




fn main() {
let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("Please provide file name as arguments");
        std::process::exit(0);
    }

    if !std::path::Path::new(&args[1]).exists() {
        println!("file {} doesn't exist!", args[1]);
        std::process::exit(0);
    }

    let mut editor = TextEditor::new(&args[1]);
    editor.run();
}




