use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

pub struct HighLighter {
    ps: SyntaxSet,
    ts: ThemeSet,
    extension: String,
}

impl HighLighter {
    pub fn new(name: &str) -> Self {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let split: Vec<&str> = name.split(".").collect();
        assert!(split.len() > 1);
        Self {
            ps,
            extension: split.last().unwrap().to_string(),
            ts,
        }
    }

    pub fn highlight_line(&self, line: &str) -> String {
        let syntax = self.ps.find_syntax_by_extension(&self.extension).unwrap();
        let theme = self.ts.themes["base16-ocean.dark"].clone();
        let mut h = HighlightLines::new(syntax, &theme);
        let ranges = h.highlight_line(line, &self.ps).unwrap();
        as_24_bit_terminal_escaped(&ranges[..], false)
    }
}
