use serde::Serialize;

#[derive(Serialize)]
pub struct CodeMap {
    filenames: Vec<String>,
    line_entries: Vec<LineEntry>,
}

#[derive(Serialize)]
pub struct LineEntry {
    filename_index: usize,
    line: usize,
}

impl CodeMap {
    pub fn new() -> Self {
        Self {
            filenames: Vec::<String>::new(),
            line_entries: Vec::<LineEntry>::new(),
        }
    }
}
