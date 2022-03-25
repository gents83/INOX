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

#[inline]
pub const fn generate_static_uid_from_string(string: &str) -> Uid {
    let bytes = string.as_bytes();
    let mut bytes_to_use: [u8; 16] = [0u8; 16];
    let mut i = 0;
    while i < 16 {
        bytes_to_use[i] = bytes[i % bytes.len()];
        i += 1;
    }
    Uuid::from_bytes(bytes_to_use)
}
