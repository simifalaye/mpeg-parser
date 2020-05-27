use std::fmt;
pub mod pat;

pub enum Psi {
    Pat(pat::Pat),
}

impl fmt::Display for Psi {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Psi::Pat(p) => write!(f, "{}", p),
        }
    }
}
