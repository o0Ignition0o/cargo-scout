pub mod rust;

pub trait Config {
    fn get_members(&self) -> Vec<String>;
}
