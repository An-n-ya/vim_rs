mod mode;
mod text;

use std::{env::args, fs, io::{stdout, stdin, Write, Stdout, BufWriter}, cell::RefCell, vec, process::Stdio};
use termion::{color, style, raw::{IntoRawMode, RawTerminal}, input::{TermRead, MouseTerminal}, event::Key, screen::AlternateScreen};
use text::Text;
use crate::mode::Mode;

struct Coordinates {
    pub x: usize,
    pub y: usize
}

enum CursorStyle {
    Bar,
    Block,
    Underline
}

struct Size(u16, u16);

struct TextEditor {
    text: Text,
    text_length: usize,
    cur_pos: Coordinates,
    cur_line: usize,
    upper_line: usize,
    lower_line: usize,
    terminal_size: Size,
    file_name: String,
    out: Box<dyn Write>,
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
            text_length,
            cur_pos: Coordinates{x:1,y:1},
            cur_line: 1,
            lower_line: 0,
            upper_line: text_length.min(size.1.into()) - 1,
            terminal_size: Size(size.0, size.1),
            file_name: file_name.into(),
            out,
        }
    }

    pub fn new_from_vec(lines: &Vec<String>) -> Self {
        let mut text = Text::new();
        for line in lines {
            text.push_line(line.clone());
        }
        let text_length = lines.len();
        let size = termion::terminal_size().unwrap();
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
            text_length,
            cur_pos: Coordinates{x:1,y:1},
            cur_line: 1,
            lower_line: 0,
            upper_line: text_length.min(size.1.into()) - 1,
            terminal_size: Size(size.0, size.1),
            file_name: "test_file".into(),
            out,
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
        for line in self.lower_line..self.upper_line {
            writeln!(self.out, "{}\r", self.text.line_at(line as usize)).unwrap();
        }
    }

    fn max_y(&self) -> u16 {
        self.terminal_size.1 - 1
    }

    fn show_bar(&mut self) {
        write!(self.out, "{}",termion::cursor::Goto(0, (self.terminal_size.1) as u16)).unwrap();
        write!(self.out, "{}{} line-count={} filename: {} {}-{} {}:{}{}",
                    color::Fg(color::Blue),
                    style::Bold,
                    self.text_length,
                    self.file_name,
                    self.lower_line,
                    self.upper_line,
                    self.cur_pos.x,
                    self.cur_pos.y,
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

    fn len_of_cur_line(&self) -> usize {
        assert!(self.cur_line != 0);
        self.text.len_of_line_at(self.cur_line - 1)
    }

    fn move_to_end_of_line(&mut self) {
        self.cur_pos.x = self.len_of_cur_line();
        self.flush();
    }
    fn move_to_start_of_line(&mut self) {
        self.cur_pos.x = 0;
        self.flush();
    }
    fn inc_x(&mut self) {
        if self.cur_pos.x < self.len_of_cur_line() {
            self.cur_pos.x += 1;
        }
        self.flush();
    }
    fn dec_x(&mut self) {
        if self.cur_pos.x > 1 {
            self.cur_pos.x -= 1;
        }
        self.flush();
    }
    fn inc_y(&mut self) {
        if self.cur_pos.y < self.max_y().into() {
            self.cur_pos.y += 1;
        } else {
            if self.upper_line < self.text_length {
                self.lower_line += 1;
                self.upper_line += 1;
            }
        }
        if self.cur_line < self.text_length {
            self.cur_line += 1;
        }
        self.flush();
    }
    fn dec_y(&mut self) {
        if self.cur_pos.y > 1 {
            self.cur_pos.y -= 1;
        } else {
            if self.lower_line > 0 {
                self.lower_line -= 1;
                self.upper_line -= 1;
            }

        }
        if self.cur_line > 1 {
            self.cur_line -= 1;
        }
        self.flush();
    }
    fn run(&mut self) {
        self.flush();
        self.out.flush().unwrap();
        let stdin = stdin();
        let mut mode = Mode::Normal;
        for c in stdin.keys() {
            mode = mode.handle(self, c.unwrap());
            if mode == Mode::Exit {
                break;
            }
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




