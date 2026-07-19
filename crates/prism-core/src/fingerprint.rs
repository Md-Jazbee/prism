//! XXH3 content hashing and directory Merkle fingerprints.

use xxhash_rust::xxh3::xxh3_128;

/// Hex-encode XXH3-128 of arbitrary bytes.
pub fn hash_bytes(bytes: &[u8]) -> String {
    hex::encode(xxh3_128(bytes).to_be_bytes())
}

/// Hash file contents (entire buffer). Empty files have a defined hash.
pub fn file_content_hash(contents: &[u8]) -> String {
    hash_bytes(contents)
}

/// Combine child hashes into a parent Merkle node.
///
/// Contract: sorted `(name, hash)` pairs; changing any leaf invalidates ancestors.
pub fn merkle_combine(children: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let mut pairs: Vec<(String, String)> = children
        .iter()
        .map(|(n, h)| (n.as_ref().to_string(), h.as_ref().to_string()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut buf = String::new();
    for (name, hash) in pairs {
        buf.push_str(&name);
        buf.push('\0');
        buf.push_str(&hash);
        buf.push('\n');
    }
    hash_bytes(buf.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_bytes_same_hash() {
        assert_eq!(file_content_hash(b"hello"), file_content_hash(b"hello"));
        assert_ne!(file_content_hash(b"hello"), file_content_hash(b"world"));
    }

    #[test]
    fn merkle_order_independent_of_input_order() {
        let a = merkle_combine(&[("b", "1"), ("a", "2")]);
        let b = merkle_combine(&[("a", "2"), ("b", "1")]);
        assert_eq!(a, b);
    }

    #[test]
    fn merkle_sensitive_to_child_change() {
        let a = merkle_combine(&[("a", "1"), ("b", "2")]);
        let b = merkle_combine(&[("a", "1"), ("b", "3")]);
        assert_ne!(a, b);
    }
}
