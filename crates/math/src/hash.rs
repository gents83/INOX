use crate::Vector3;

fn murmur_finalize_32(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x85ebca6b);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;
    hash
}

fn murmur_32(list: &[u32]) -> u32 {
    let mut hash = 0;
    for i in list {
        let mut element = *i;
        element = element.wrapping_mul(0xcc9e2d51);
        element = (element << 15) | (element >> (32 - 15));
        element = element.wrapping_mul(0x1b873593);

        hash ^= element;
        hash = (hash << 13) | (hash >> (32 - 13));
        hash = hash.wrapping_mul(5);
        hash = hash.wrapping_add(0xe6546b64);
    }
    murmur_finalize_32(hash)
}

pub fn compute_hash_position(p: &Vector3) -> u32 {
    murmur_32(&[
        if p.x == 0. { 0 } else { p.x.to_bits() },
        if p.y == 0. { 0 } else { p.y.to_bits() },
        if p.z == 0. { 0 } else { p.z.to_bits() },
    ])
}
