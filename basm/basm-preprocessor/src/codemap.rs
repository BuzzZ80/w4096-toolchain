use serde::Serialize;

#[derive(Serialize)]
pub struct CodeMap {
    pub filenames: Vec<String>,
    pub line_entries: Vec<LineEntry>,
}

#[derive(Serialize)]
pub struct LineEntry {
    pub filename_index: usize,
    pub line: usize,
}

impl CodeMap {
    pub fn new() -> Self {
        Self {
            filenames: Vec::<String>::new(),
            line_entries: Vec::<LineEntry>::new(),
        }
    }
}
