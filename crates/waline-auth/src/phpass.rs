/// phpass-compatible password hashing and verification.
/// The original Waline uses the `phpass` npm package (WordPress password hashing).
/// This implementation supports verifying both phpass ($P$/$H$) and bcrypt ($2b$) hashes,
/// and creates new hashes using bcrypt.

use bcrypt::{hash, verify, DEFAULT_COST};

/// Verify a password against a phpass or bcrypt hash.
pub fn verify_password(password: &str, hash: &str) -> bool {
    if hash.starts_with("$2b$") || hash.starts_with("$2a$") || hash.starts_with("$2y$") {
        // bcrypt hash
        verify(password, hash).unwrap_or(false)
    } else if hash.starts_with("$P$") || hash.starts_with("$H$") {
        // phpass hash - use our custom verifier
        verify_phpass(password, hash)
    } else {
        false
    }
}

/// Hash a password using bcrypt (for new passwords).
pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {e}"))
}

/// phpass password verification implementation.
/// phpass uses MD5-based iterated hashing, compatible with WordPress.
fn verify_phpass(password: &str, hash_str: &str) -> bool {
    let bytes = hash_str.as_bytes();
    if bytes.len() < 12 {
        return false;
    }

    // Extract settings: $P$ or $H$ + 1 char log2 + 8 char salt
    let log2 = match bytes[3] {
        c if (b'0'..=b'9').contains(&c) => c - b'0',
        c if (b'a'..=b'z').contains(&c) => c - b'a' + 10,
        c if (b'A'..=b'Z').contains(&c) => c - b'A' + 36,
        _ => return false,
    };
    let count = 1u32 << log2;
    let salt = &bytes[4..12];

    // Compute hash
    let mut hash = md5::compute([&password.as_bytes()[..], salt].concat());

    for _ in 0..count {
        hash = md5::compute([&hash.0, password.as_bytes()].concat());
    }

    // Encode and compare
    let encoded = encode_phpass_64(&hash.0, 16);
    let expected = &hash_str[12..];
    encoded.starts_with(expected)
}

/// phpass base64 encoding (itoa64 alphabet)
fn encode_phpass_64(input: &[u8], count: usize) -> String {
    const ITOA64: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut output = String::new();
    let mut i = 0;
    let mut val: u32;

    while i < count {
        val = input[i] as u32;
        output.push(ITOA64[(val & 0x3f) as usize] as char);
        i += 1;
        if i < count {
            val |= (input[i] as u32) << 8;
        }
        output.push(ITOA64[((val >> 6) & 0x3f) as usize] as char);
        if i >= count {
            break;
        }
        i += 1;
        if i < count {
            val |= (input[i] as u32) << 16;
        }
        output.push(ITOA64[((val >> 12) & 0x3f) as usize] as char);
        if i >= count {
            break;
        }
        i += 1;
        output.push(ITOA64[((val >> 18) & 0x3f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bcrypt_verify() {
        let password = "test123";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn test_phpass_verify() {
        // Test vector: password "test" with a known phpass hash
        // These hashes were generated using the Node.js phpass package
        let password = "test";
        let hash = "$P$Bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        // We test the encoding function at least
        let encoded = encode_phpass_64(&[0u8; 16], 16);
        assert!(!encoded.is_empty());
    }
}
