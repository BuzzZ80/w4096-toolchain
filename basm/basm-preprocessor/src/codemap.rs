use serde::Serialize;

#[derive(Serialize)]
pub struct CodeMap {
    pub filenames: Vec<String>,
    pub line_entries: Vec<LineEntry>,
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
    pub fn add_entry(&mut self, filename_index: usize, line: usize) {
        self.line_entries.push(LineEntry {
            filename_index,
            line,
        });
    }
    pub fn push(&mut self, other: &Self) {
        let offset = self.filenames.len();  // How much to add to each filename index

        // Go through modifying and adding each line entry
        for entry in &other.line_entries {
            self.line_entries.push(LineEntry{
                filename_index: entry.filename_index + offset,  // Add offset for new filenames
                line: entry.line,                               // Keep same line as before :)
            });
        };

        self.filenames.extend_from_slice(other.filenames.as_slice());
    }
}
