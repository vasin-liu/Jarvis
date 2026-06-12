use sha2::{Digest, Sha256};

pub fn hash_text(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    hex::encode(digest)
}
