use termion::event::Key;

use crate::TextEditor;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Normal,
    Visual,
    Insert,
    Command,
    Exit
}

impl Mode {
    pub fn handle(&self,editor: &mut TextEditor, key: Key) -> Self {
        match self {
            Mode::Normal => Self::handle_normal(editor, key),
            Mode::Visual => Self::handle_visual(editor, key),
            Mode::Insert => Self::handle_insert(editor, key),
            Mode::Command => Self::handle_insert(editor, key),
            Mode::Exit => unreachable!(),
        }
    }

    fn handle_normal(editor: &mut TextEditor, key: Key) -> Self {
        match key {
                Key::Ctrl('q') => {
                    Mode::Exit
                },
                Key::Char('h') | Key::Left => {
                    editor.dec_x();
                    Mode::Normal
                },
                Key::Char('j') | Key::Down => {
                    editor.inc_y();
                    Mode::Normal
                },
                Key::Char('k') | Key::Up => {
                    editor.dec_y();
                    Mode::Normal
                },
                Key::Char('l') | Key::Right => {
                    editor.inc_x();
                    Mode::Normal
                },
                Key::Char('A') => {
                    editor.change_mode_immediately(Mode::Insert);
                    editor.move_to_end_of_line();
                    editor.set_cursor_style(crate::CursorStyle::Bar);
                    Mode::Insert
                },
                Key::Char('$') => {
                    editor.move_to_end_of_line();
                    Mode::Normal
                },
                Key::Char('0') => {
                    editor.move_to_start_of_line();
                    Mode::Normal
                },
                Key::Char('e') => {
                    editor.forward_to_end_of_next_word();
                    Mode::Normal
                },
                Key::Char('w') => {
                    editor.forward_to_start_of_next_word();
                    Mode::Normal
                },
                Key::Char('b') => {
                    editor.backward_to_start_of_next_word();
                    Mode::Normal
                },
                Key::Char('u') => {
                    todo!();
                    Mode::Normal
                },
                Key::Char('a') => {
                    editor.change_mode_immediately(Mode::Insert);
                    editor.inc_x();
                    editor.set_cursor_style(crate::CursorStyle::Bar);
                    Mode::Insert
                },
                Key::Backspace => {
                    editor.backward_to_next_char();
                    editor.flush();
                    Mode::Normal
                },
                Key::Char('i') => {
                    editor.set_cursor_style(crate::CursorStyle::Bar);
                    Mode::Insert
                },
                Key::Char(':') => {
                    Mode::Command
                },
                Key::Char('v') => {
                    Mode::Visual
                },
                _ => Mode::Normal
        }
    }

    fn handle_visual(editor: &mut TextEditor, key: Key) -> Self {
        match key {
            Key::Esc => {
                editor.set_cursor_style(crate::CursorStyle::Block);
                Mode::Normal
            },
            Key::Ctrl('q') => {
                Mode::Exit
            },
            _ => Mode::Visual
        }
    }
    fn handle_insert(editor: &mut TextEditor, key: Key) -> Self {

        match key {
            Key::Char(c) => {
                if c == '\n' {
                    // TODO: considering expand the upper
                    return Mode::Insert;
                }
                let x = editor.cur_line - 1;
                let y = editor.cur_pos.x - 1;
                editor.text.insert_at(x, y, c);
                editor.inc_x();
                Mode::Insert
            },
            Key::Left => {
                editor.dec_x();
                Mode::Insert
            },
            Key::Down => {
                editor.inc_y();
                Mode::Insert
            },
            Key::Up => {
                editor.dec_y();
                Mode::Insert
            },
            Key::Right => {
                editor.inc_x();
                Mode::Insert
            },
            Key::Backspace => {
                let x = editor.cur_line - 1;
                let y = editor.cur_pos.x - 1;
                if y == 0 {
                    if x != 0 {
                        editor.delete_line_at(x);
                        editor.dec_y();
                        editor.move_to_end_of_line();
                    }
                } else {
                    editor.text.delete_at(x, y);
                    editor.dec_x();
                }
                Mode::Insert
            },
            Key::Esc => {
                if editor.cursor_at_end_of_line() {
                    editor.dec_x()
                }
                editor.set_cursor_style(crate::CursorStyle::Block);
                Mode::Normal
            },
            Key::Ctrl('q') => {
                Mode::Exit
            },
            _ => Mode::Insert
        }
    }
    fn handle_command(editor: &mut TextEditor, key: Key) -> Self {

        match key {
            Key::Esc => {
                editor.set_cursor_style(crate::CursorStyle::Block);
                Mode::Normal
            },
            Key::Ctrl('q') => {
                Mode::Exit
            },
            _ => Mode::Command
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use termion::event::Key;

    fn init(lines: Vec<String>) -> TextEditor {
        return TextEditor::new_from_vec(&lines);
    }

    fn handle_keys(editor: &mut TextEditor, keys: Vec<Key>) {
        let mut mode = Mode::Normal;
        for c in keys {
            mode = mode.handle(editor, c);
            if mode == Mode::Exit {
                break;
            }
            editor.out.flush().unwrap();
        }
    }

    #[test]
    fn basic_insert() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        let keys = vec![
            Key::Char('i'),
            Key::Char('a'),
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "ahello");

        let keys = vec![
            // FIXME: when we esc from 'i', we shouldn't use 'h'
            Key::Char('h'),
            Key::Char('a'),
            Key::Char(' '),
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "a hello");

        let keys = vec![
            Key::Char('A'),
            Key::Char(' '),
            Key::Char('t'),
            Key::Char('e'),
            Key::Char('s'),
            Key::Char('t'),
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "a hello test");
    }

    #[test]
    fn move_between_word() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        assert_eq!(editor.cur_char(), 'h');
        handle_keys(&mut editor, vec![Key::Char('e')]);
        assert_eq!(editor.cur_char(), 'o');
        handle_keys(&mut editor, vec![Key::Char('e')]);
        assert_eq!(editor.cur_char(), 'd');
        handle_keys(&mut editor, vec![Key::Char('b')]);
        assert_eq!(editor.cur_char(), 'w');
        handle_keys(&mut editor, vec![Key::Char('b')]);
        assert_eq!(editor.cur_char(), 'h');
        handle_keys(&mut editor, vec![Key::Char('w')]);
        assert_eq!(editor.cur_char(), 'w');
        handle_keys(&mut editor, vec![Key::Backspace, Key::Backspace]);
        assert_eq!(editor.cur_char(), 'l');
    }

    #[test]
    fn delete_in_insert() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        let keys = vec![
            Key::Char('j'),
            Key::Char('A'),
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "hello");
        assert_eq!(editor.text.line_at(1), "");
        let keys = vec![
            Key::Char('i'),
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Backspace,
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "");
        assert_eq!(editor.text_length(), 1);
    }
}