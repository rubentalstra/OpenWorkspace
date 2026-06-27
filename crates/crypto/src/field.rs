//! AEAD field encryption and token hashing for reversible secrets.
//!
//! Reversible secrets (the TOTP shared secret today; OIDC client secrets later)
//! are sealed with AES-256-GCM under a data-encryption key (DEK). The DEK is in
//! turn wrapped under a root key-encryption key (KEK) held in configuration —
//! the envelope scheme the `crypto_keys` table exists for. High-entropy,
//! single-use tokens (recovery codes, API keys) are *not* reversible, so they
//! are hashed with SHA-256 ([`hash_token`]) and looked up by their hash.
//!
//! Every ciphertext begins with a one-byte suite tag, so the algorithm can be
//! rotated centrally while previously written ciphertext still decrypts. The
//! matching textual identifier ([`SUITE_ID_AES_256_GCM_V1`]) is recorded in
//! `crypto_keys.suite_id`.

use std::fmt;

use aws_lc_rs::aead::{AES_256_GCM, Aad, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use aws_lc_rs::digest::{SHA256, digest};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::CryptoError;

/// AES-256 key length in bytes.
const KEY_LEN: usize = 32;

/// SHA-256 digest length in bytes.
const TOKEN_HASH_LEN: usize = 32;

/// Suite tag stamped as the first byte of every ciphertext envelope.
const SUITE_TAG_AES_256_GCM_V1: u8 = 1;

/// Textual suite identifier recorded in `crypto_keys.suite_id`.
pub const SUITE_ID_AES_256_GCM_V1: &str = "aead:aes256-gcm:v1";

/// Associated data binding a wrapped DEK to its purpose, so a wrapped key cannot
/// be replayed as field ciphertext (and vice versa).
const WRAP_AAD: &[u8] = b"crypto_keys/wrap";

/// A data-encryption key: the symmetric key that seals individual fields.
///
/// Wiped on drop. `Debug` is redacted so the bytes never reach logs.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DataKey([u8; KEY_LEN]);

impl fmt::Debug for DataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DataKey").field(&"<redacted>").finish()
    }
}

/// A root key-encryption key, supplied from configuration, that wraps DEKs.
///
/// Wiped on drop. `Debug` is redacted so the bytes never reach logs.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct RootKey([u8; KEY_LEN]);

impl fmt::Debug for RootKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RootKey").field(&"<redacted>").finish()
    }
}

impl RootKey {
    /// Build a root key from exactly [`KEY_LEN`] bytes of high-entropy material.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let key = <[u8; KEY_LEN]>::try_from(bytes).map_err(|_| CryptoError::KeyLength)?;
        Ok(Self(key))
    }
}

/// The SHA-256 hash of a high-entropy token. Storable as `bytea`, compared by
/// equality (lookups go through a unique index, never a hand-rolled compare).
#[derive(Clone, PartialEq, Eq)]
pub struct TokenHash([u8; TOKEN_HASH_LEN]);

impl TokenHash {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for TokenHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TokenHash").field(&"<redacted>").finish()
    }
}

/// Generate a fresh random data-encryption key.
pub fn generate_data_key() -> Result<DataKey, CryptoError> {
    let mut bytes = [0u8; KEY_LEN];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| CryptoError::Rng)?;
    Ok(DataKey(bytes))
}

/// Seal `plaintext` under `key`, binding `aad`. Output: `[suite tag][nonce][ct‖tag]`.
pub fn encrypt_field(key: &DataKey, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    seal(&key.0, plaintext, aad)
}

/// Open an envelope produced by [`encrypt_field`] under `key`, checking `aad`.
///
/// Returns [`CryptoError::Decrypt`] if the suite tag is unknown, the envelope is
/// truncated, the `aad` differs, or the authentication tag fails.
pub fn decrypt_field(key: &DataKey, envelope: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    open(&key.0, envelope, aad)
}

/// Wrap a DEK under the root KEK, producing the value stored in
/// `crypto_keys.wrapped_key`.
pub fn wrap_key(kek: &RootKey, dek: &DataKey) -> Result<Vec<u8>, CryptoError> {
    seal(&kek.0, &dek.0, WRAP_AAD).map_err(|_| CryptoError::KeyWrap)
}

/// Unwrap a DEK previously sealed by [`wrap_key`].
pub fn unwrap_key(kek: &RootKey, wrapped: &[u8]) -> Result<DataKey, CryptoError> {
    let mut bytes = open(&kek.0, wrapped, WRAP_AAD).map_err(|_| CryptoError::KeyWrap)?;
    let key = <[u8; KEY_LEN]>::try_from(bytes.as_slice()).map_err(|_| CryptoError::KeyLength);
    bytes.zeroize();
    Ok(DataKey(key?))
}

/// Hash a high-entropy token (recovery code, API key) for storage and lookup.
///
/// Not for passwords — those are salted and slow-hashed via
/// [`crate::hash_password`]. A token is high-entropy, so a fast unsalted SHA-256
/// is sufficient and lets the stored hash be found by a unique index.
#[must_use]
pub fn hash_token(token: &[u8]) -> TokenHash {
    let computed = digest(&SHA256, token);
    let mut bytes = [0u8; TOKEN_HASH_LEN];
    bytes.copy_from_slice(computed.as_ref());
    TokenHash(bytes)
}

fn seal(key_bytes: &[u8; KEY_LEN], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let unbound = UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|_| CryptoError::Encrypt)?;
    let key = LessSafeKey::new(unbound);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    SystemRandom::new()
        .fill(&mut nonce_bytes)
        .map_err(|_| CryptoError::Rng)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::from(aad), &mut in_out)
        .map_err(|_| CryptoError::Encrypt)?;

    let mut envelope = Vec::with_capacity(1 + NONCE_LEN + in_out.len());
    envelope.push(SUITE_TAG_AES_256_GCM_V1);
    envelope.extend_from_slice(&nonce_bytes);
    envelope.append(&mut in_out);
    Ok(envelope)
}

fn open(key_bytes: &[u8; KEY_LEN], envelope: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let (&tag, rest) = envelope.split_first().ok_or(CryptoError::Decrypt)?;
    if tag != SUITE_TAG_AES_256_GCM_V1 || rest.len() < NONCE_LEN {
        return Err(CryptoError::Decrypt);
    }
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes).map_err(|_| CryptoError::Decrypt)?;

    let unbound = UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|_| CryptoError::Decrypt)?;
    let key = LessSafeKey::new(unbound);

    let mut in_out = ciphertext.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::from(aad), &mut in_out)
        .map_err(|_| CryptoError::Decrypt)?;
    Ok(plaintext.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dek() -> DataKey {
        generate_data_key().unwrap()
    }

    #[test]
    fn encrypt_decrypt_round_trips() {
        let key = dek();
        let plaintext = b"super-secret-totp-seed";
        let aad = b"totp_secret";
        let envelope = encrypt_field(&key, plaintext, aad).unwrap();
        assert_eq!(decrypt_field(&key, &envelope, aad).unwrap(), plaintext);
    }

    #[test]
    fn envelope_is_tagged_and_nonced() {
        let envelope = encrypt_field(&dek(), b"x", b"").unwrap();
        assert_eq!(envelope[0], SUITE_TAG_AES_256_GCM_V1);
        // suite tag + 12-byte nonce + 1 byte ciphertext + 16-byte GCM tag.
        assert_eq!(envelope.len(), 1 + NONCE_LEN + 1 + 16);
    }

    #[test]
    fn fresh_nonce_makes_ciphertext_non_deterministic() {
        let key = dek();
        let a = encrypt_field(&key, b"same", b"").unwrap();
        let b = encrypt_field(&key, b"same", b"").unwrap();
        assert_ne!(a, b, "a random nonce must vary the ciphertext");
    }

    #[test]
    fn tampered_ciphertext_is_rejected() {
        let key = dek();
        let mut envelope = encrypt_field(&key, b"data", b"aad").unwrap();
        let last = envelope.len() - 1;
        envelope[last] ^= 0x01;
        assert_eq!(decrypt_field(&key, &envelope, b"aad"), Err(CryptoError::Decrypt));
    }

    #[test]
    fn wrong_aad_is_rejected() {
        let key = dek();
        let envelope = encrypt_field(&key, b"data", b"context-a").unwrap();
        assert_eq!(
            decrypt_field(&key, &envelope, b"context-b"),
            Err(CryptoError::Decrypt)
        );
    }

    #[test]
    fn wrong_key_is_rejected() {
        let envelope = encrypt_field(&dek(), b"data", b"").unwrap();
        assert_eq!(decrypt_field(&dek(), &envelope, b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn unknown_suite_tag_is_rejected() {
        let key = dek();
        let mut envelope = encrypt_field(&key, b"data", b"").unwrap();
        envelope[0] = 0xFF;
        assert_eq!(decrypt_field(&key, &envelope, b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn truncated_envelope_is_rejected() {
        let key = dek();
        assert_eq!(decrypt_field(&key, &[], b""), Err(CryptoError::Decrypt));
        assert_eq!(decrypt_field(&key, &[SUITE_TAG_AES_256_GCM_V1], b""), Err(CryptoError::Decrypt));
    }

    #[test]
    fn wrap_unwrap_round_trips_a_dek() {
        let kek = RootKey::from_bytes(&[7u8; KEY_LEN]).unwrap();
        let dek = generate_data_key().unwrap();
        let wrapped = wrap_key(&kek, &dek).unwrap();

        // The unwrapped DEK must decrypt what the original DEK sealed.
        let envelope = encrypt_field(&dek, b"payload", b"aad").unwrap();
        let unwrapped = unwrap_key(&kek, &wrapped).unwrap();
        assert_eq!(decrypt_field(&unwrapped, &envelope, b"aad").unwrap(), b"payload");
    }

    #[test]
    fn unwrap_with_wrong_kek_fails() {
        let kek = RootKey::from_bytes(&[1u8; KEY_LEN]).unwrap();
        let other = RootKey::from_bytes(&[2u8; KEY_LEN]).unwrap();
        let wrapped = wrap_key(&kek, &generate_data_key().unwrap()).unwrap();
        assert!(matches!(unwrap_key(&other, &wrapped), Err(CryptoError::KeyWrap)));
    }

    #[test]
    fn root_key_rejects_wrong_length() {
        assert!(matches!(
            RootKey::from_bytes(&[0u8; 16]),
            Err(CryptoError::KeyLength)
        ));
        assert!(RootKey::from_bytes(&[0u8; KEY_LEN]).is_ok());
    }

    #[test]
    fn hash_token_is_deterministic_and_distinct() {
        assert_eq!(hash_token(b"abc"), hash_token(b"abc"));
        assert_ne!(hash_token(b"abc"), hash_token(b"abd"));
        assert_eq!(hash_token(b"abc").as_bytes().len(), TOKEN_HASH_LEN);
    }

    #[test]
    fn key_debug_is_redacted() {
        let dbg = format!("{:?}", dek());
        assert!(dbg.contains("redacted"));
    }
}
