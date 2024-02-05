use termion::event::Key;

use crate::TextEditor;

#[derive(Default)]
pub struct Task {
    tasks: Vec<Key>
}

impl Task {
    pub fn push(&mut self, key: Key)  {
        self.tasks.push(key);
    }
    pub fn perform_task(&mut self, editor: &mut TextEditor) {

    }
}