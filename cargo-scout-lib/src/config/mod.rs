pub mod rust;

pub trait Config {
    fn linter_must_iterate(&self) -> bool;
    fn get_members(&self) -> Vec<String>;
}
