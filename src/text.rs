pub struct Text {
    lines: Vec<String>
}

impl Text {
    pub fn new() -> Self {
        Self {
            lines: vec![]
        }
    }
    pub fn insert_at(&mut self, x: usize, y: usize, c: char) {
        let x = x.min(self.lines.len() - 1);
        let y = y.min(self.lines[x].len());
        #[cfg(test)]
        println!("insert at x={x}, y={y}");
        self.lines[x].insert(y, c)
    }

    pub fn line_at(&self, line: usize) -> String {
        let line = line.min(self.lines.len() - 1);
        self.lines[line].clone()
    }

    pub fn push_line(&mut self, content: String) {
        self.lines.push(content);
    }

    // idx start from 0
    pub fn add_line_before(&mut self, idx: usize, content: String) {
        if idx > self.lines.len() {
            return self.push_line(content);
        }
        self.lines.insert(idx, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_basic() {
        let lines = vec!["hello".to_string(), "world".to_string()];
        let mut text = Text{lines};
        for (i, c) in "Annya ".chars().enumerate() {
            text.insert_at(0, i, c);
        }
        for (i, c) in " and happy every day!".chars().enumerate() {
            text.insert_at(10, i + 10, c);
        }
        assert_eq!(text.line_at(0), "Annya hello".to_string());
        assert_eq!(text.line_at(1), "world and happy every day!".to_string());
    }
}