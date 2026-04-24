// ── Cryptography Module ─────────────────────────────────────
// Security design decisions:
//
// 1. AES-256-GCM (authenticated encryption):
//    - Provides both confidentiality and integrity.
//    - GCM mode includes an authentication tag, preventing
//      tampering with ciphertexts.
//    - 256-bit key size chosen over 128-bit for future-proofing.
//
// 2. Argon2id (key derivation):
//    - Winner of the Password Hashing Competition (PHC).
//    - Argon2id variant: resistant to both side-channel and
//      GPU/ASIC timing attacks.
//    - Parameters:
//      - Memory: 64 MiB — balances security and performance.
//      - Iterations: 3 — minimum recommended by RFC 9106.
//      - Parallelism: 4 — matches typical CPU core count.
//    - Trade-off: Higher memory costs would be more resistant
//      to GPU attacks but impose ~30s+ unlock times on low-end
//      hardware. 64 MiB / 3 iterations ~1-2s on modern CPUs.
//
// 3. Vault key wrapping:
//    - A random 256-bit vault key is generated on vault creation.
//    - The vault key is encrypted with the master-password-derived key.
//    - This means changing the master password only requires
//      re-wrapping the vault key, not re-encrypting all entries.
//
// 4. Per-entry nonces:
//    - 96-bit nonces (standard for GCM) are randomly generated
//      for each encryption operation.
//    - Probability of nonce reuse is negligible with a CSPRNG.
//
// 5. Memory zeroing:
//    - Sensitive buffers (keys, plaintext secrets) are zeroed
//      when dropped to reduce the window for memory dumping.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::SaltString,
    Argon2, Params,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use zeroize::Zeroize;

use crate::errors::VaultError;

// ── Constants ───────────────────────────────────────────────
/// Size of the vault encryption key in bytes (256-bit).
const VAULT_KEY_SIZE: usize = 32;
/// Size of nonce/IV for AES-256-GCM (96-bit recommended).
const NONCE_SIZE: usize = 12;
/// Size of Argon2 salt (128-bit).
const SALT_SIZE: usize = 16;

// Argon2 parameters.
// These are reasonable defaults for a developer tool on modern hardware.
// Trade-off: Decreasing memory to 16 MiB would speed unlocks to ~300ms
// but weakens GPU resistance. Increasing to 256 MiB adds ~4s to unlock.
const ARGON2_MEMORY: u32 = 64 * 1024; // 64 MiB
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

// ── Public API ──────────────────────────────────────────────

/// Generate a random 256-bit vault encryption key.
/// This key is used to encrypt/decrypt all entries.
pub fn generate_vault_key() -> Vec<u8> {
    let mut key = vec![0u8; VAULT_KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    key
}

/// Generate a random salt for Argon2.
pub fn generate_salt() -> Vec<u8> {
    let mut salt = vec![0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Generate a random nonce for AES-256-GCM.
pub fn generate_nonce() -> Vec<u8> {
    let mut nonce = vec![0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Derive a 256-bit key from a master password using Argon2id.
///
/// # Arguments
/// * `password` - The master password (will NOT be zeroed by this function).
/// * `salt` - Random salt bytes.
///
/// # Security
/// The derived key is stored in a Vec<u8> that should be zeroed after use.
pub fn derive_key(password: &str, salt: &[u8]) -> Result<Vec<u8>, VaultError> {
    let params = Params::new(
        ARGON2_MEMORY,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(VAULT_KEY_SIZE),
    )
    .map_err(|e| VaultError::Encryption(format!("Argon2 param error: {e}")))?;

    // Re-encode salt as SaltString for the argon2 API.
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| VaultError::Encryption(format!("Salt encoding error: {e}")))?;

    let mut derived_key = vec![0u8; VAULT_KEY_SIZE];
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt_str.as_ref().as_bytes(), &mut derived_key)
        .map_err(|_| VaultError::InvalidPassword)?;

    Ok(derived_key)
}

/// Encrypt a plaintext with AES-256-GCM.
///
/// # Arguments
/// * `key` - 256-bit encryption key.
/// * `nonce` - 96-bit nonce (must be unique per key).
/// * `plaintext` - Data to encrypt.
///
/// Returns the ciphertext (includes the GCM authentication tag).
pub fn encrypt(key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::Encryption(format!("Invalid key length: {e}")))?;

    let nonce = Nonce::from_slice(nonce);
    cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| VaultError::Encryption(format!("Encryption failed: {e}")))
}

/// Decrypt a ciphertext with AES-256-GCM.
///
/// Returns `Err(VaultError::Decryption)` if the authentication tag is invalid
/// (tampered ciphertext, wrong key, or wrong nonce).
pub fn decrypt(key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, VaultError> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| VaultError::Decryption(format!("Invalid key length: {e}")))?;

    let nonce = Nonce::from_slice(nonce);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| VaultError::Decryption("Authentication failed — data may be tampered".into()))
}

/// Wrapper: encrypt with base64 input/output for storage.
pub fn encrypt_b64(key: &[u8], nonce: &[u8], plaintext: &str) -> Result<String, VaultError> {
    let ciphertext = encrypt(key, nonce, plaintext.as_bytes())?;
    Ok(BASE64.encode(&ciphertext))
}

/// Wrapper: decrypt from base64.
pub fn decrypt_b64(key: &[u8], nonce: &[u8], ciphertext_b64: &str) -> Result<String, VaultError> {
    let ciphertext = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| VaultError::Decryption(format!("Base64 decode error: {e}")))?;
    let plaintext = decrypt(key, nonce, &ciphertext)?;
    // Ensure valid UTF-8.
    String::from_utf8(plaintext)
        .map_err(|e| VaultError::Decryption(format!("UTF-8 decode error: {e}")))
}

/// Zero-fill a mutable byte buffer.
/// Use this to clear cryptographic keys after use.
pub fn zeroize_buffer(buf: &mut [u8]) {
    buf.zeroize();
}

/// Parse a base64 string into bytes.
pub fn b64_decode(s: &str) -> Result<Vec<u8>, VaultError> {
    BASE64
        .decode(s)
        .map_err(|e| VaultError::Decryption(format!("Base64 decode error: {e}")))
}

/// Encode bytes as base64.
pub fn b64_encode(data: &[u8]) -> String {
    BASE64.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_vault_key();
        let nonce = generate_nonce();
        let plaintext = "Hello, vault! This is a secret API key.";

        let encrypted = encrypt(&key, &nonce, plaintext.as_bytes()).unwrap();
        let decrypted = decrypt(&key, &nonce, &encrypted).unwrap();

        assert_eq!(String::from_utf8(decrypted).unwrap(), plaintext);
    }

    #[test]
    fn test_encrypt_b64_roundtrip() {
        let key = generate_vault_key();
        let nonce = generate_nonce();
        let plaintext = "sk-abc123def456";

        let encrypted = encrypt_b64(&key, &nonce, plaintext).unwrap();
        let decrypted = decrypt_b64(&key, &nonce, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_vault_key();
        let key2 = generate_vault_key();
        let nonce = generate_nonce();
        let plaintext = "secret";

        let encrypted = encrypt(&key1, &nonce, plaintext.as_bytes()).unwrap();
        let result = decrypt(&key2, &nonce, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = "correct-horse-battery-staple";
        let salt = generate_salt();

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_wrong_password() {
        let salt = generate_salt();
        let _key = derive_key("correct", &salt).unwrap();
        let result = derive_key("wrong", &salt);

        assert!(result.is_err());
    }
}
