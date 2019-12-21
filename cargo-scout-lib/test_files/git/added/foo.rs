#[derive(Debug, PartialEq)]
pub struct Section {
    pub file_name: String,
    pub line_start: u32,
    pub line_end: u32,
}
