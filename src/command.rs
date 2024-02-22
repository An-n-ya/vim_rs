use termion::event::Key;

use crate::Coordinates;

#[derive(Clone)]
pub struct CmdAction {
    pub action: Action,
    pub pos: Coordinates,
    pub cur_line: usize,
    pub contents: Vec<Key>,
}

#[derive(Clone)]
pub enum Action {
    Insert,
    Delete,
}

#[derive(Default)]
pub struct ActionStack {
    backward_stack: Vec<CmdAction>,
    forward_stack: Vec<CmdAction>,
}

impl ActionStack {
    pub fn forward(&mut self) -> Option<CmdAction> {
        let action = self.forward_stack.pop();
        if action.is_none() {
            return None;
        }

        let action = action.unwrap();
        self.backward_stack.push(action.clone());
        Some(action)
    }

    pub fn backward(&mut self) -> Option<CmdAction> {
        let action = self.backward_stack.pop();
        if action.is_none() {
            return None;
        }

        let action = action.unwrap();
        self.forward_stack.push(action.clone());
        Some(action)
    }

    pub fn discard_key_on_top(&mut self) {
        let idx = self.backward_stack.len() - 1;
        self.backward_stack[idx].contents.pop();
    }
    pub fn append_key_to_top(&mut self, key: Key) {
        let idx = self.backward_stack.len() - 1;
        self.backward_stack[idx].contents.push(key)
    }
    pub fn append_string_to_top(&mut self, s: String) {
        for c in s.chars() {
            self.append_key_to_top(Key::Char(c));
        }
    }

    pub fn add_action(&mut self, action: Action, cur_line: usize, pos: Coordinates) {
        self.backward_stack.push(CmdAction {
            action,
            cur_line,
            pos,
            contents: vec![],
        })
    }
}
