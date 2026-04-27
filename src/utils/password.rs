use crate::error::AppError;

/// Hash a password with bcrypt (cost 10).
pub fn hash(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, 10).map_err(|e| AppError::Internal(e.to_string()))
}

/// Verify a password against a stored hash.
/// Supports bcrypt ($2b$, $2a$, $2y$) and phpass ($P$, $H$).
pub fn verify(password: &str, hash: &str) -> Result<bool, AppError> {
    if hash.starts_with("$P$") || hash.starts_with("$H$") {
        Ok(verify_phpass(password, hash))
    } else {
        bcrypt::verify(password, hash).map_err(|e| AppError::Internal(e.to_string()))
    }
}

/// Minimal phpass portable hash verification.
fn verify_phpass(password: &str, hash: &str) -> bool {
    const ITOA64: &[u8] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

    if hash.len() != 34 {
        return false;
    }
    let count_log2 = ITOA64.iter().position(|&b| b == hash.as_bytes()[3]).unwrap_or(0);
    let count = 1usize << count_log2;
    let salt = &hash[4..12];

    let mut h = md5::compute(format!("{salt}{password}").as_bytes()).to_vec();
    for _ in 0..count {
        let mut input = h.clone();
        input.extend_from_slice(password.as_bytes());
        h = md5::compute(&input).to_vec();
    }

    let encoded = encode64(&h, 16, ITOA64);
    &hash[12..] == &encoded[..22]
}

fn encode64(input: &[u8], count: usize, itoa64: &[u8]) -> String {
    let mut output = String::new();
    let mut i = 0;
    while i < count {
        let mut value = input[i] as usize;
        i += 1;
        output.push(itoa64[value & 0x3f] as char);
        if i < count {
            value |= (input[i] as usize) << 8;
        }
        output.push(itoa64[(value >> 6) & 0x3f] as char);
        if i >= count {
            break;
        }
        i += 1;
        if i < count {
            value |= (input[i] as usize) << 16;
        }
        output.push(itoa64[(value >> 12) & 0x3f] as char);
        if i >= count {
            break;
        }
        i += 1;
        output.push(itoa64[(value >> 18) & 0x3f] as char);
    }
    output
}
