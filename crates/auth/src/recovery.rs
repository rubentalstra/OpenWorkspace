//! One-time recovery codes — a fallback second factor when an authenticator is
//! lost. Codes are high-entropy, shown once, and stored only as SHA-256 hashes
//! (via the `crypto` facade); a used code is consumed in the database.

use crypto::hash_token;
use rand::Rng;

use crate::AuthError;

/// How many codes a freshly issued set contains.
pub const RECOVERY_CODE_COUNT: usize = 10;
/// Characters per code, excluding the group separator.
const CODE_CHARS: usize = 10;
/// Crockford-style base32 alphabet, omitting ambiguous I/L/O/U.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A freshly generated recovery-code set: the plaintext to show the user once and
/// the storage hashes. The two vectors are positionally aligned.
pub struct RecoveryCodes {
    /// Human-readable codes (`XXXXX-XXXXX`), shown to the user exactly once.
    pub plaintext: Vec<String>,
    /// SHA-256 hashes (32 bytes each) to persist.
    pub hashes: Vec<Vec<u8>>,
}

impl std::fmt::Debug for RecoveryCodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveryCodes")
            .field("count", &self.plaintext.len())
            .finish_non_exhaustive()
    }
}

/// Generate a fresh set of [`RECOVERY_CODE_COUNT`] recovery codes.
///
/// # Errors
///
/// Never fails today; returns [`Result`] so a future entropy source can.
pub fn generate_recovery_codes() -> Result<RecoveryCodes, AuthError> {
    let mut rng = rand::rng();
    let mut plaintext = Vec::with_capacity(RECOVERY_CODE_COUNT);
    let mut hashes = Vec::with_capacity(RECOVERY_CODE_COUNT);
    for _ in 0..RECOVERY_CODE_COUNT {
        let code = generate_one(&mut rng);
        hashes.push(hash_submitted_code(&code));
        plaintext.push(code);
    }
    Ok(RecoveryCodes { plaintext, hashes })
}

/// Hash a recovery code for storage or lookup, canonicalising first so that case
/// and separators do not matter.
#[must_use]
pub fn hash_submitted_code(input: &str) -> Vec<u8> {
    hash_token(canonical(input).as_bytes()).as_bytes().to_vec()
}

fn generate_one(rng: &mut impl Rng) -> String {
    let mut code = String::with_capacity(CODE_CHARS + 1);
    for i in 0..CODE_CHARS {
        if i == CODE_CHARS / 2 {
            code.push('-');
        }
        let idx = rng.random_range(0..ALPHABET.len());
        code.push(char::from(ALPHABET[idx]));
    }
    code
}

fn canonical(code: &str) -> String {
    code.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn generates_the_expected_count_and_shape() {
        let set = generate_recovery_codes().unwrap();
        assert_eq!(set.plaintext.len(), RECOVERY_CODE_COUNT);
        assert_eq!(set.hashes.len(), RECOVERY_CODE_COUNT);
        for code in &set.plaintext {
            assert_eq!(code.len(), CODE_CHARS + 1, "XXXXX-XXXXX");
            assert!(code.contains('-'));
        }
        // Hashes are 32-byte SHA-256 digests, and all codes are unique.
        assert!(set.hashes.iter().all(|h| h.len() == 32));
        let unique: HashSet<_> = set.plaintext.iter().collect();
        assert_eq!(unique.len(), RECOVERY_CODE_COUNT);
    }

    #[test]
    fn canonicalisation_ignores_case_and_separators() {
        let set = generate_recovery_codes().unwrap();
        let code = &set.plaintext[0];
        let messy = format!("  {}  ", code.to_lowercase());
        assert_eq!(hash_submitted_code(code), hash_submitted_code(&messy));
        assert_eq!(hash_submitted_code(code), set.hashes[0]);
    }

    #[test]
    fn distinct_codes_hash_differently() {
        assert_ne!(hash_submitted_code("AAAAA-BBBBB"), hash_submitted_code("AAAAA-BBBBC"));
    }
}
