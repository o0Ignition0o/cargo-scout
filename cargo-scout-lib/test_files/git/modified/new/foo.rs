#[derive(Debug, PartialEq)]
pub struct Section {
    pub file_names: String,
    pub line_start: u32,
    pub line_end: u32,
    pub more_foo: Option<i32>,
}
