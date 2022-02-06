use uuid::Uuid;

pub type Uid = Uuid;
pub const INVALID_UID: Uid = Uuid::nil();

#[inline]
pub fn generate_random_uid() -> Uid {
    Uuid::new_v4()
}
#[inline]
pub fn generate_uid_from_string(string: &str) -> Uid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, string.as_bytes())
}
