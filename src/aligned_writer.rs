use std::fmt::Display;
use std::fmt::Write;

pub(crate) struct AlignedWriter {
    lines: Vec<String>,
}

impl AlignedWriter {
    pub(crate) fn new(count: usize) -> AlignedWriter {
        AlignedWriter {
            lines: vec![String::new(); count],
        }
    }

    pub(crate) fn write_n_l(&mut self, s: impl IntoIterator<Item = impl Display>) {
        let words = s.into_iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(self.lines.len(), words.len());
        let max_width = words.iter().map(|s| s.len()).max().unwrap_or(0);
        for (line, word) in self.lines.iter_mut().zip(words) {
            write!(line, "{:<w$}", word, w = max_width).unwrap();
        }
    }

    pub(crate) fn write_n_r(&mut self, s: impl IntoIterator<Item = impl Display>) {
        let words = s.into_iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(self.lines.len(), words.len());
        let max_width = words.iter().map(|s| s.len()).max().unwrap_or(0);
        for (line, word) in self.lines.iter_mut().zip(words) {
            write!(line, "{:>w$}", word, w = max_width).unwrap();
        }
    }

    pub(crate) fn write(&mut self, s: impl Display) {
        let len = self.lines.len();
        self.write_n_l(std::iter::repeat(&s).take(len));
    }

    pub(crate) fn print(&self) {
        for line in &self.lines {
            println!("{}", line);
        }
    }
}
