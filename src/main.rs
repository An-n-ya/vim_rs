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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    select_view: SelectView,
    terminal_size: Size,
    file_name: String,
    out: Box<dyn Write>,
    mode: Mode,
    task: Task,
    action_stack: ActionStack,
    processing_action: bool,
    processing_task: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum SelectView {
    CharacterView(CharacterView),
    LineView(LineView),
    BlockView(CharacterView),
    None
}

#[derive(Debug, PartialEq, Eq)]
struct CharacterView {
    start: Coordinates,
    end: Coordinates,
}
#[derive(Debug, PartialEq, Eq)]
struct LineView {
    start: usize,
    end: usize,
}

#[derive(Debug)]
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
            select_view: SelectView::None,
            terminal_size: Size(size.0, size.1),
            file_name: file_name.into(),
            out,
            mode: Mode::Normal,
            task: Task::default(),
            action_stack: ActionStack::default(),
            processing_action: false,
            processing_task: false,
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
            select_view: SelectView::None,
            terminal_size: Size(size.0, size.1),
            file_name: "test_file".into(),
            out,
            mode: Mode::Normal,
            task: Task::default(),
            action_stack: ActionStack::default(),
            processing_action: false,
            processing_task: false,
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
            let text = self.text.line_at(line as usize);
            for (col, c) in text.chars().enumerate() {
                if self.is_select_start(col, line) {
                    write!(self.out, "{}", termion::style::Invert).unwrap();
                }
                write!(self.out, "{}", c).unwrap();
                if self.is_select_end(col, line) {
                    write!(self.out, "{}", termion::style::NoInvert).unwrap();
                }
            }
            writeln!(self.out, "\r").unwrap();
        }
    }

    fn is_select_end(&mut self, col: usize, line: usize) -> bool {
        match Self::sort_select_view(&self.select_view) {
            SelectView::CharacterView(v) => {
                line > v.end.y || col >= v.end.x && line == v.end.y
            },
            SelectView::LineView(v) => {
                col >= v.end
            },
            SelectView::BlockView(v) => {
                col == v.end.x && line <= v.end.y
            },
            SelectView::None => false,
        }
    }
    fn is_select_start(&mut self, col: usize, line: usize) -> bool {
        match Self::sort_select_view(&self.select_view) {
            SelectView::CharacterView(v) => {
                (line > v.start.y || col >= v.start.x && line == v.start.y)
                && (line < v.end.y || line == v.end.y && col <= v.end.x)
            },
            SelectView::LineView(v) => {
                line >= v.start && line <= v.end
            },
            SelectView::BlockView(v) => {
                col == v.start.x && line >= v.start.y
            },
            SelectView::None => false,
        }
    }

    fn sort_select_view(mode: &SelectView) -> SelectView {
        match mode {
            SelectView::CharacterView(v) => {
                let mut start = v.start;
                let mut end = v.end;
                if end.y < start.y || start.y == end.y && end.x < start.x {
                    (start, end) = (end, start);
                }
                SelectView::CharacterView(CharacterView{start, end})
            },
            SelectView::LineView(v) => {
                let mut start = v.start;
                let mut end = v.end;
                if end < start {
                    (start, end) = (end, start);
                }
                SelectView::LineView(LineView{start, end})
            },
            SelectView::BlockView(_) => todo!(),
            SelectView::None => SelectView::None,
        }
    }

    pub fn set_visual_mode(&mut self, mode: SelectView) {
        self.select_view = mode;
    }

    pub fn update_visual_pos(&mut self) {
        if self.mode != Mode::Visual {
            return;
        }
        match &self.select_view {
            SelectView::CharacterView(v) => {
                let mut end = self.cur_pos;
                end = Coordinates{x: end.x - 1, y: self.cur_line - 1};
                let start = v.start;


                self.select_view = SelectView::CharacterView(CharacterView{start, end});
            },
            SelectView::LineView(v) => {
                let start = v.start;
                self.select_view = SelectView::LineView(LineView{start, end: self.cur_line - 1});
            },
            SelectView::BlockView(_) => todo!(),
            SelectView::None => return,
        }
    }

    fn max_y(&self) -> u16 {
        self.terminal_size.1 - 1
    }

    fn show_bar(&mut self) {
        write!(self.out, "{}",termion::cursor::Goto(0, (self.terminal_size.1) as u16)).unwrap();
        write!(self.out, "{}{} line-count={} filename: {}, size: ({}, {}) line[{}-{}] pos[{}:{}] mode:{} task:{} {}",
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
                    self.task,
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
            // FIXME: cursor is not blinking
            CursorStyle::Bar => write!(self.out, "{}", termion::cursor::BlinkingBar),
            CursorStyle::Block => write!(self.out, "{}", termion::cursor::BlinkingBlock),
            CursorStyle::Underline => write!(self.out, "{}", termion::cursor::BlinkingUnderline),
        }.unwrap();
    }

    fn update_pos(&mut self) {
        write!(self.out, "{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16)).unwrap();
    }

    pub fn try_perform_task(&mut self) {
        self.processing_task = true;
        if self.task.is_movement() {
            // it is guaranteed that current tasks have num
            assert!(self.task.has_num());
            let n = self.task.num().unwrap();
            let key = *self.task.last_task().unwrap();
            for _ in 0..n {
                Mode::handle_normal(self, key);
            }
            self.task.clear();
        } else if self.task.last_two_task() == Some("dd".to_string()) {
            // FIXME: considering `2dd`
            self.delete_cur_line();
            self.task.clear();
        }
        self.processing_task = false;
    }

    pub fn revoke_action(&mut self, action: Option<CmdAction>) {
        self.processing_action = true;

        if let Some(action) = action {
            let pos = action.pos;
            let cur_line = action.cur_line;
            self.set_pos(pos.x, pos.y);
            self.cur_line = cur_line;
            match action.action {
                Action::Delete => {
                    action.contents.iter().for_each(|&a| {
                        match a {
                            Key::Char(c) => self.insert_char_at_cur(c),
                            _ => unreachable!()
                        }
                    })
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

        self.processing_action = false;
    }

    pub fn restore_action(&mut self, action: Option<CmdAction>)  {
        self.processing_action = true;
        if let Some(action) = action {
            let pos = action.pos;
            let cur_line = action.cur_line;
            self.set_pos(pos.x, pos.y);
            self.cur_line = cur_line;
            match action.action {
                Action::Insert => {
                    action.contents.iter().for_each(|&a| {
                        if cfg!(test) {
                            println!("restoring insert key:{:?}", a);
                        }
                        Mode::handle_insert(self, a);
                    })
                },
                Action::Delete => {
                    action.contents.iter().for_each(|&_a| {
                        // consider restoring `dd`
                        Mode::handle_normal(self, Key::Char('x'));
                    })

                },
            }
        }
        self.processing_action = false;

    }

    fn len_of_cur_line(&self) -> usize {
        assert!(self.cur_line != 0);
        if self.mode == Mode::Normal || self.mode == Mode::Visual || self.mode == Mode::Command {
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

    pub fn delete_cur_char(&mut self) -> Option<char> {
        if self.cur_char() == 0 as char {
            if self.cur_line < self.text_length() {
                let contents = self.delete_line_at(self.cur_line);
                self.text.append_str_at(self.cur_line - 1, self.len_of_cur_line(), contents);
            }
            None
        } else {
            self.text.delete_at(self.cur_line - 1, self.cur_pos.x)
        }
    }

    fn insert_char_at_cur(&mut self, c: char) {
        self.insert_char_at(c, self.cur_line - 1, self.cur_pos.x - 1);
    }

    fn insert_char_at(&mut self, c: char, x: usize, y: usize) {
        if c == '\n' {
            self.new_line();
        } else {
            self.text.insert_at(x, y, c)
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




