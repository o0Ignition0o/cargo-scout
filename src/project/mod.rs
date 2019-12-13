pub mod cargo;

pub trait Config {
    fn linter_must_iterate(&self) -> bool;
    fn get_members(&self) -> Vec<String>;
}
