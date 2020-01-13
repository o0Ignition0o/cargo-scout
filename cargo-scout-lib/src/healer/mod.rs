use crate::linter::Lint;

pub trait Healer {
    fn heal(&self, lints: Vec<Lint>) -> Result<(), crate::error::Error>;
}
