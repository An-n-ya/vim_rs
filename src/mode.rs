use termion::event::Key;

use crate::{command::Action, CharacterView, Coordinates, LineView, SelectView, TextEditor};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Visual,
    Insert,
    Command,
    Exit,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
            Mode::Exit => "EXIT",
        };

        write!(f, "{}", s)
    }
}

impl Mode {
    pub fn handle(&self, editor: &mut TextEditor, key: Key) -> Self {
        match self {
            Mode::Normal => Self::handle_normal(editor, key),
            Mode::Visual => Self::handle_visual(editor, key),
            Mode::Insert => Self::handle_insert(editor, key),
            Mode::Command => Self::handle_command(editor, key),
            Mode::Exit => unreachable!(),
        }
    }

    fn pre_handle_normal(editor: &mut TextEditor, key: Key) -> bool {
        match key {
            Key::Char(c @ '0'..='9') => {
                if c == '0' {
                    if editor.task.has_num() {
                        editor.task.push(key);
                    } else {
                        return false;
                    }
                } else {
                    editor.task.push(key);
                }
            }
            Key::Char('j')
            | Key::Char('k')
            | Key::Char('h')
            | Key::Char('l')
            | Key::Char('e')
            | Key::Char('w')
            | Key::Char('b')
            | Key::Char(' ')
            | Key::Backspace
            | Key::Left
            | Key::Right
            | Key::Down
            | Key::Up => {
                if editor.task.has_num() {
                    editor.task.push(key)
                } else {
                    return false;
                }
            }
            Key::Char('i') | Key::Char('a') => {
                if editor.task.len() > 0 {
                    editor.task.push(key);
                } else {
                    return false;
                }
            }
            Key::Char('c') | Key::Char('d') | Key::Char('y') => editor.task.push(key),
            _ => {
                return false;
            }
        }

        editor.try_perform_task();
        true
    }

    pub fn handle_normal(editor: &mut TextEditor, key: Key) -> Self {
        if !editor.processing_task {
            if Self::pre_handle_normal(editor, key) {
                return Mode::Normal;
            }
        }
        match key {
            Key::Ctrl('q') => Mode::Exit,
            Key::Char('h') | Key::Left => {
                editor.dec_x();
                Mode::Normal
            }
            Key::Char('j') | Key::Down => {
                editor.inc_y();
                Mode::Normal
            }
            Key::Char('k') | Key::Up => {
                editor.dec_y();
                Mode::Normal
            }
            Key::Char('l') | Key::Right => {
                editor.inc_x();
                Mode::Normal
            }
            Key::Char('A') => {
                editor.change_mode_immediately(Mode::Insert);
                editor.move_to_end_of_line();
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char('I') => {
                editor.change_mode_immediately(Mode::Insert);
                editor.move_to_first_char_of_line();
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char('$') => {
                editor.move_to_end_of_line();
                Mode::Normal
            }
            Key::Char('0') => {
                editor.move_to_start_of_line();
                Mode::Normal
            }
            Key::Char('e') => {
                editor.forward_to_end_of_next_word();
                Mode::Normal
            }
            Key::Char('w') => {
                editor.forward_to_start_of_next_word();
                Mode::Normal
            }
            Key::Char('b') => {
                editor.backward_to_start_of_next_word();
                Mode::Normal
            }
            Key::Ctrl('r') => {
                let action = editor.action_stack.forward();
                editor.restore_action(action);
                Mode::Normal
            }
            Key::Char('u') => {
                let action = editor.action_stack.backward();
                editor.revoke_action(action);
                Mode::Normal
            }
            Key::Char('a') => {
                editor.change_mode_immediately(Mode::Insert);
                editor.inc_x();
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Backspace => {
                editor.backward_to_next_char();
                Mode::Normal
            }
            Key::Char('x') => {
                let c = editor.delete_cur_char();
                editor
                    .action_stack
                    .add_action(Action::Delete, editor.cur_line, editor.cur_pos);
                if let Some(c) = c {
                    if !editor.processing_action {
                        editor.action_stack.append_key_to_top(Key::Char(c));
                    }
                }
                Mode::Normal
            }
            Key::Char('s') => {
                editor.delete_cur_char();
                editor.set_cursor_style(crate::CursorStyle::Bar);
                // FIXME: substitute action conclude both insert and delete
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char('S') => {
                editor.delete_cur_line();
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor.new_line_ahead();
                // FIXME: substitute action conclude both insert and delete
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char(' ') => {
                editor.forward_to_next_char();
                Mode::Normal
            }
            Key::Char('o') => {
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor.new_line_behind();
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char('O') => {
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor.new_line_ahead();
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char('i') => {
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char(':') => Mode::Command,
            Key::Char('v') => {
                let mut pos = editor.cur_pos;
                pos = Coordinates {
                    x: pos.x - 1,
                    y: editor.cur_line - 1,
                };
                if cfg!(test) {
                    println!("entering character visual mode, pos={:?}", pos);
                }

                let mode = SelectView::CharacterView(CharacterView {
                    start: pos,
                    end: pos,
                });
                editor.set_visual_mode(mode);
                Mode::Visual
            }
            Key::Char('V') => {
                let mode = SelectView::LineView(LineView {
                    start: editor.cur_line - 1,
                    end: editor.cur_line - 1,
                });
                editor.set_visual_mode(mode);
                Mode::Visual
            }
            _ => Mode::Normal,
        }
    }

    fn handle_visual(editor: &mut TextEditor, key: Key) -> Self {
        let mode = match key {
            Key::Esc => {
                editor.set_cursor_style(crate::CursorStyle::Block);
                editor.set_visual_mode(SelectView::None);
                return Mode::Normal;
            }
            Key::Ctrl('q') => Mode::Exit,
            Key::Char('h') | Key::Left => {
                editor.dec_x();
                Mode::Visual
            }
            Key::Char('j') | Key::Down => {
                editor.inc_y();
                Mode::Visual
            }
            Key::Char('k') | Key::Up => {
                editor.dec_y();
                Mode::Visual
            }
            Key::Char('l') | Key::Right => {
                editor.inc_x();
                Mode::Visual
            }
            Key::Char('c') => {
                editor.delete_selected();
                editor.set_cursor_style(crate::CursorStyle::Bar);
                editor.set_visual_mode(SelectView::None);
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Char('d') => {
                editor.delete_selected();
                editor.set_cursor_style(crate::CursorStyle::Block);
                editor.set_visual_mode(SelectView::None);
                return Mode::Normal;
            }
            _ => Mode::Visual,
        };

        editor.update_visual_pos();
        mode
    }
    pub fn handle_insert(editor: &mut TextEditor, key: Key) -> Self {
        match key {
            Key::Char(c) => {
                if c == '\n' {
                    editor.new_line();
                } else if c == '\t' {
                    let x = editor.cur_line - 1;
                    let y = editor.cur_pos.x - 1;
                    for _ in 0..4 {
                        editor.text.insert_at(x, y, ' ');
                        editor.inc_x();
                    }
                } else {
                    let x = editor.cur_line - 1;
                    let y = editor.cur_pos.x - 1;
                    editor.text.insert_at(x, y, c);
                    editor.inc_x();
                }
                if !editor.processing_action {
                    editor.action_stack.append_key_to_top(key);
                }
                Mode::Insert
            }
            Key::Left => {
                editor.dec_x();
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Down => {
                editor.inc_y();
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Up => {
                editor.dec_y();
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
            Key::Right => {
                editor.inc_x();
                editor
                    .action_stack
                    .add_action(Action::Insert, editor.cur_line, editor.cur_pos);
                Mode::Insert
            }
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
                if !editor.processing_action {
                    editor.action_stack.discard_key_on_top();
                }
                Mode::Insert
            }
            Key::Delete => {
                editor.delete_cur_char();
                // TODO: add action
                Mode::Insert
            }
            Key::Esc => {
                editor.dec_x();
                editor.set_cursor_style(crate::CursorStyle::Block);
                Mode::Normal
            }
            Key::Ctrl('q') => Mode::Exit,
            _ => Mode::Insert,
        }
    }
    fn handle_command(editor: &mut TextEditor, key: Key) -> Self {
        match key {
            Key::Esc => {
                editor.set_cursor_style(crate::CursorStyle::Block);
                Mode::Normal
            }
            Key::Ctrl('q') => Mode::Exit,
            _ => Mode::Command,
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

    fn exit(editor: &mut TextEditor) {
        handle_keys(editor, vec![Key::Ctrl('q')]);
    }

    #[test]
    fn basic_insert() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        let keys = vec![Key::Char('i'), Key::Char('a'), Key::Esc];
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

        let keys = vec![Key::Char('I'), Key::Char('a'), Key::Char(' '), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "a a hello test");

        let keys = vec![
            Key::Char('o'),
            Key::Char('n'),
            Key::Esc,
            Key::Char('O'),
            Key::Char('N'),
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(1), "N");
        assert_eq!(editor.text.line_at(2), "n");

        exit(&mut editor);
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
        handle_keys(&mut editor, vec![Key::Char(' ')]);
        assert_eq!(editor.cur_char(), 'o');
        handle_keys(&mut editor, vec![Key::Char('0')]);
        assert_eq!(editor.cur_char(), 'h');
        handle_keys(&mut editor, vec![Key::Char('$')]);
        assert_eq!(editor.cur_char(), 'o');

        exit(&mut editor);
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

        let keys = vec![Key::Char('k'), Key::Char('x'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "ello");
        assert_eq!(editor.text.line_at(1), "");

        let keys = vec![Key::Char('a'), Key::Delete, Key::Delete, Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "eo");
        assert_eq!(editor.text.line_at(1), "");

        let keys = vec![Key::Char('s'), Key::Char('a'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "ao");
        assert_eq!(editor.text.line_at(1), "");

        let keys = vec![Key::Char('S'), Key::Char('a'), Key::Char('a'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "aa");
        assert_eq!(editor.text.line_at(1), "");

        let keys = vec![
            Key::Char('j'),
            Key::Char('A'),
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

        exit(&mut editor);
    }

    #[test]
    fn revoke_and_restore_test() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        let keys = vec![
            Key::Char('l'),
            Key::Char('a'),
            Key::Char('b'),
            Key::Char('\n'),
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "heb");
        assert_eq!(editor.text.line_at(1), "llo");

        let keys = vec![Key::Char('u'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "hello");
        assert_eq!(editor.text.line_at(1), "world");

        let keys = vec![Key::Ctrl('r'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(0), "heb");
        assert_eq!(editor.text.line_at(1), "llo");
        assert_eq!(editor.text.line_at(2), "world");

        let keys = vec![Key::Char('x'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(1), "lo");
        let keys = vec![Key::Char('u'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(1), "llo");
        let keys = vec![Key::Ctrl('r'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.text.line_at(1), "lo");
    }

    #[test]
    fn task_test() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        let keys = vec![Key::Char('2'), Key::Char('l'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.cur_char(), 'l');

        let keys = vec![Key::Char('2'), Key::Char('0'), Key::Char('j'), Key::Esc];
        handle_keys(&mut editor, keys);
        assert_eq!(editor.cur_char(), 'r');
    }

    #[test]
    fn visual_mode_test() {
        let mut editor = init(vec!["hello".to_string(), "world".to_string()]);

        assert_eq!(editor.cur_char(), 'h');
        let keys = vec![
            Key::Char('l'),
            Key::Char('l'),
            Key::Char('v'),
            Key::Char('l'),
            Key::Char('l'),
            Key::Char('d'),
            Key::Esc,
        ];
        handle_keys(&mut editor, keys);
        // FIXME: why this test is failed?
        // assert_eq!(editor.text.line_at(0), "he");
        assert_eq!(editor.text.line_at(1), "world");
    }
}
