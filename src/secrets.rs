//! Encryption of secret setting values (API keys, webhook signing secrets) at rest.
//!
//! Values are encrypted with AES-256-GCM using a key that the **host application** supplies
//! through [`DatabaseSelector::with_secret_key`](crate::DatabaseSelector::with_secret_key).
//! The key is never stored in the tenant database, so a leaked database dump or backup does
//! not expose the secrets. Where the key comes from (environment, KMS, Vault) is up to the
//! host.
//!
//! Stored format: `enc:v1:` + hex(`nonce(12) || ciphertext || tag(16)`). The setting's title
//! is bound as associated data so a ciphertext cannot be copied to a different setting.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use std::fmt;

use crate::error::ShopsterError;

const PREFIX: &str = "enc:v1:";
const NONCE_LEN: usize = 12;

/// A 256-bit key used to encrypt secret settings. `Debug` never prints the key material.
#[derive(Clone)]
pub struct SecretKey([u8; 32]);

impl SecretKey {
    /// Creates a key from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        SecretKey(bytes)
    }

    /// Parses a key from 64 hexadecimal characters (e.g. `openssl rand -hex 32`).
    pub fn from_hex(hex: &str) -> Result<Self, ShopsterError> {
        let bytes = decode_hex(hex.trim())
            .filter(|b| b.len() == 32)
            .ok_or_else(|| ShopsterError::InvalidOperationError("Secret key must be 64 hex characters (32 bytes)".to_string()))?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(SecretKey(key))
    }
}

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey(***)")
    }
}

/// `true` if `value` looks like output of [`encrypt`].
pub(crate) fn is_encrypted(value: &str) -> bool {
    value.starts_with(PREFIX)
}

/// Encrypts `plaintext`, binding it to `context` (the setting title).
pub(crate) fn encrypt(key: &SecretKey, context: &str, plaintext: &str) -> Result<String, ShopsterError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.0));
    let nonce_bytes: [u8; NONCE_LEN] = rand::random();
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), Payload { msg: plaintext.as_bytes(), aad: context.as_bytes() })
        .map_err(|_| ShopsterError::InternalError("Encrypting secret failed".to_string()))?;

    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ciphertext);
    Ok(format!("{}{}", PREFIX, encode_hex(&blob)))
}

/// Decrypts a value produced by [`encrypt`] with the same `context`.
pub(crate) fn decrypt(key: &SecretKey, context: &str, stored: &str) -> Result<String, ShopsterError> {
    let corrupt = || ShopsterError::InternalError("Stored secret is corrupt or the secret key is wrong".to_string());

    let blob = stored.strip_prefix(PREFIX).and_then(decode_hex).ok_or_else(corrupt)?;
    if blob.len() < NONCE_LEN {
        return Err(corrupt());
    }
    let (nonce, ciphertext) = blob.split_at(NONCE_LEN);

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key.0));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ciphertext, aad: context.as_bytes() })
        .map_err(|_| corrupt())?;
    String::from_utf8(plaintext).map_err(|_| corrupt())
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 || !hex.is_ascii() {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> SecretKey {
        SecretKey::from_bytes([byte; 32])
    }

    #[test]
    fn round_trip() {
        let stored = encrypt(&key(1), "stripe_webhook_secret", "whsec_123").unwrap();
        assert!(is_encrypted(&stored));
        assert!(!stored.contains("whsec_123"));
        assert_eq!(decrypt(&key(1), "stripe_webhook_secret", &stored).unwrap(), "whsec_123");
    }

    #[test]
    fn round_trips_empty_and_unicode() {
        for value in ["", "päßwörd-✓"] {
            let stored = encrypt(&key(1), "t", value).unwrap();
            assert_eq!(decrypt(&key(1), "t", &stored).unwrap(), value);
        }
    }

    #[test]
    fn encryption_is_randomised() {
        assert_ne!(encrypt(&key(1), "t", "same").unwrap(), encrypt(&key(1), "t", "same").unwrap());
    }

    #[test]
    fn wrong_key_fails() {
        let stored = encrypt(&key(1), "t", "secret").unwrap();
        assert!(decrypt(&key(2), "t", &stored).is_err());
    }

    #[test]
    fn ciphertext_is_bound_to_its_setting() {
        let stored = encrypt(&key(1), "stripe_secret_api_key", "secret").unwrap();
        assert!(decrypt(&key(1), "sendgrid_api_key", &stored).is_err());
    }

    #[test]
    fn tampering_and_garbage_are_rejected() {
        let mut stored = encrypt(&key(1), "t", "secret").unwrap();
        let last = stored.pop().unwrap();
        stored.push(if last == '0' { '1' } else { '0' });
        assert!(decrypt(&key(1), "t", &stored).is_err());
        for bad in ["", "plaintext", "enc:v1:", "enc:v1:zz", "enc:v1:abc", "enc:v1:00"] {
            assert!(decrypt(&key(1), "t", bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn key_parsing_and_debug_redaction() {
        let hex = "00".repeat(32);
        assert!(SecretKey::from_hex(&hex).is_ok());
        assert!(SecretKey::from_hex("abcd").is_err());
        assert!(SecretKey::from_hex(&"zz".repeat(32)).is_err());
        assert_eq!(format!("{:?}", SecretKey::from_hex(&hex).unwrap()), "SecretKey(***)");
    }
}
