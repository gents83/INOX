pub trait Handle
{
    fn is_valid(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrustedHandle;
