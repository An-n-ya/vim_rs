use crate::Coordinates;

pub struct Text {
    lines: Vec<String>,
}

impl Text {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }
    pub fn char_at(&mut self, x: usize, y: usize) -> char {
        if x >= self.lines.len() || self.lines[x].len() == 0 || y >= self.lines[x].len() {
            return 0 as char;
        }
        self.lines[x].chars().nth(y).unwrap()
    }
    pub fn insert_at(&mut self, x: usize, y: usize, c: char) {
        let x = x.min(self.lines.len() - 1);
        let y = y.min(self.lines[x].len());
        #[cfg(test)]
        println!("insert c={c} at x={x}, y={y}");
        self.lines[x].insert(y, c)
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn delete_line_at(&mut self, x: usize) -> String {
        let x = x.min(self.lines.len() - 1);
        self.lines.remove(x)
    }
    pub fn append_str_at(&mut self, x: usize, y: usize, s: String) {
        let x = x.min(self.lines.len() - 1);
        let y = y.min(self.lines[x].len());
        self.lines[x].insert_str(y, &s);
    }
    pub fn delete_at(&mut self, x: usize, y: usize) -> Option<char> {
        let x = x.min(self.lines.len() - 1);
        let y = y.min(self.lines[x].len());
        let y = 0.max(y - 1);
        if self.lines[x].len() > 0 {
            return Some(self.lines[x].remove(y));
        }
        None
    }
    pub fn delete_range(&mut self, start: Coordinates, end: Coordinates) -> String {
        let former: String;
        let latter: String;
        let mut deleted: String = String::new();
        if start.x == end.x {
            assert!(start.y <= end.y);
            let text = self.lines[start.x].clone();
            former = text[0..start.y].to_string();
            latter = text[end.y + 1..].to_string();
            deleted = text[start.y..end.y + 1].to_string();
        } else {
            assert!(start.x < end.x);
            for i in (start.x + 1..end.x).into_iter().rev() {
                deleted.push_str(&self.delete_line_at(i));
                deleted.push('\n');
            }
            former = (&self.lines[start.x].clone()[0..start.y]).to_string();
            latter = (&self.lines[start.x + 1].clone()[end.y + 1..]).to_string();
            deleted.push_str(&self.lines[start.x][start.y..]);
            deleted.push('\n');
            deleted.push_str(&self.lines[start.x + 1][..end.y + 1]);
            self.delete_line_at(start.x + 1);
        }
        if former.len() == 0 && latter.len() == 0 {
            self.delete_line_at(start.x);
            deleted.push('\n');
        } else {
            self.lines[start.x] = former + &latter;
        }
        deleted
    }

    pub fn len_of_line_at(&self, line: usize) -> usize {
        let line = line.min(self.lines.len() - 1);
        self.lines[line].len()
    }
    pub fn line_at(&self, line: usize) -> String {
        if line >= self.lines.len() {
            return "".to_string();
        }
        self.lines[line].clone()
    }

    pub fn new_line_at(&mut self, x: usize, index: usize) {
        let x = x.min(self.lines.len() - 1);
        let index = index.min(self.lines[x].len());
        let latter = self.lines[x][index..].to_string();
        self.lines[x].truncate(index);
        self.add_line_before(x + 1, latter);
    }

    pub fn push_line(&mut self, content: String) {
        self.lines.push(content);
    }

    pub fn pop_char_at_line(&mut self, line: usize) -> Option<char> {
        self.lines[line].pop()
    }
    pub fn push_char_at_line(&mut self, line: usize, c: char) {
        if self.lines.len() == 0 {
            self.lines.push("".to_string());
        }
        self.lines[line].push(c)
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
        let mut text = Text { lines };
        for (i, c) in "Annya ".chars().enumerate() {
            text.insert_at(0, i, c);
        }
        for (i, c) in " and happy every day!".chars().enumerate() {
            text.insert_at(10, i + 10, c);
        }
        assert_eq!(text.line_at(0), "Annya hello".to_string());
        assert_eq!(text.line_at(1), "world and happy every day!".to_string());
        for i in 0..21 {
            text.delete_at(1, 6 + i);
        }
        assert_eq!(text.line_at(1), "world".to_string());
        text.append_str_at(1, 5, " and happy everyday!".to_string());
        assert_eq!(text.line_at(1), "world and happy everyday!".to_string());
        text.delete_line_at(1);
        assert_eq!(text.len(), 1);
        assert_eq!(text.char_at(1, 0), 0 as char);
    }

    #[test]
    fn new_line() {
        let lines = vec!["hello".to_string(), "world".to_string()];
        let mut text = Text { lines };
        text.new_line_at(1, 2);
        assert_eq!(text.line_at(1), "wo");
        assert_eq!(text.line_at(2), "rld");
    }
}
