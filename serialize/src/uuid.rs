use uuid::*;

pub type UID = Uuid;
pub const INVALID_UID: UID = Uuid::nil();

pub fn generate_random_uid() -> UID {
    Uuid::new_v4()
}
pub fn generate_uid_from_string(string: &str) -> UID {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, string.as_bytes())
}
