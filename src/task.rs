use termion::event::Key;

#[derive(Default)]
pub struct Task {
    tasks: Vec<Key>
}

impl Task {
    const MOVEMENT: [Key; 13] = [
        Key::Char('j'),Key::Char('k'),Key::Char('h'),Key::Char('l'),
        Key::Char('e'),Key::Char('w'),Key::Char('b'),
        Key::Char(' '),Key::Backspace,
        Key::Left,Key::Right,Key::Down,Key::Up,
        ];
    pub fn push(&mut self, key: Key)  {
        self.tasks.push(key);
    }
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    pub fn last_task(&self) -> Option<&Key> {
        self.tasks.last()
    }
    pub fn has_num(&self) -> bool {
        let mut res = false;
        self.iter(|c| {
            if c.is_numeric() {
                res = true;
            }
        });
        res
    }
    pub fn num(&self) -> Option<usize> {
        if !self.has_num() {
            return None;
        }
        let mut s = "".to_string();
        self.iter(|c| {
            if c.is_numeric() {
                s.push(c);
            }
        });
        usize::from_str_radix(&s, 10).ok()
    }
    pub fn clear(&mut self) {
        self.tasks.clear();
    }
    pub fn is_movement(&self) -> bool {
        if let Some(key) = self.last_task() {
            return Self::MOVEMENT.contains(key);
        }
        false
    }

    fn iter<F>(&self, mut f: F) where F: FnMut(char) -> () {

        for task in &self.tasks {
            match task {
                Key::Char(c) => {
                    f(*c);
                },
                _ => {}
            };
        }

    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = "".to_string();
        self.iter(|c| {
            s.push(c);
        });
        write!(f, "{}", s)
    }
}