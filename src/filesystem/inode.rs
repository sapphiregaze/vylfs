use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Generates a truncated, non-zero inode number hash based on the given path.
pub fn generate(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);

    let full_hash = hasher.finish();

    (full_hash & 0x0FFF_FFFF) + 1
}
