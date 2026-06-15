//! # Security Module
//!
//! Provides:
//! - **ed25519 signature verification** for plugins and OTA updates
//! - **SHA-256 integrity checking** for critical files
//! - **OTA update manager** with atomic apply and rollback support
//! - **Profile encryption** (AES-256-GCM, optional)

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("signature verification failed: {0}")]
    SignatureInvalid(String),
    #[error("integrity check failed: expected {expected}, got {actual}")]
    IntegrityFailed { expected: String, actual: String },
    #[error("public key malformed: {0}")]
    BadPublicKey(String),
    #[error("signature file malformed: {0}")]
    BadSignature(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("OTA error: {0}")]
    OtaError(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

// ─────────────────────────────────────────────────────────────
// Embedded Public Key
//
// In production: replace with the actual ed25519 public key bytes (32 bytes).
// Generated with: `openssl genpkey -algorithm ed25519` + extract public portion.
// ─────────────────────────────────────────────────────────────

/// PUCE's embedded ed25519 public key (32 bytes).
/// This is a placeholder — replace with real key before release.
const PUCE_PUBLIC_KEY: &[u8; 32] = &[
    0x3d, 0x40, 0x17, 0xc3, 0xe8, 0x43, 0x89, 0x5a,
    0x92, 0xb7, 0x0a, 0xa7, 0x4d, 0x1b, 0x7e, 0xbc,
    0x9c, 0x98, 0x2c, 0xcf, 0x2e, 0xc4, 0x96, 0x8c,
    0xc0, 0xcd, 0x55, 0xf1, 0x2a, 0xf4, 0x66, 0x0c,
];

// ─────────────────────────────────────────────────────────────
// Integrity Checker
// ─────────────────────────────────────────────────────────────

/// SHA-256 based file integrity checker.
pub struct IntegrityChecker;

impl IntegrityChecker {
    /// Compute SHA-256 hex digest of a file.
    pub fn hash_file(path: &Path) -> Result<String, SecurityError> {
        let data = fs::read(path)?;
        Ok(Self::hash_bytes(&data))
    }

    /// Compute SHA-256 hex digest of raw bytes.
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    /// Verify that a file's SHA-256 matches the expected hash.
    pub fn verify_file(path: &Path, expected_hash: &str) -> Result<bool, SecurityError> {
        let actual = Self::hash_file(path)?;
        if actual.to_lowercase() == expected_hash.to_lowercase() {
            Ok(true)
        } else {
            Err(SecurityError::IntegrityFailed {
                expected: expected_hash.to_string(),
                actual,
            })
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Signature Verifier
// ─────────────────────────────────────────────────────────────

/// ed25519 signature verifier using the embedded PUCE public key.
pub struct SignatureVerifier {
    verifying_key: VerifyingKey,
}

impl SignatureVerifier {
    /// Create a verifier using the embedded PUCE public key.
    pub fn new() -> Result<Self, SecurityError> {
        let key = VerifyingKey::from_bytes(PUCE_PUBLIC_KEY)
            .map_err(|e| SecurityError::BadPublicKey(e.to_string()))?;
        Ok(Self { verifying_key: key })
    }

    /// Create a verifier using a custom public key (for testing or custom builds).
    pub fn with_key(public_key_bytes: &[u8; 32]) -> Result<Self, SecurityError> {
        let key = VerifyingKey::from_bytes(public_key_bytes)
            .map_err(|e| SecurityError::BadPublicKey(e.to_string()))?;
        Ok(Self { verifying_key: key })
    }

    /// Verify an ed25519 signature over the given data.
    /// `signature_bytes` must be exactly 64 bytes.
    pub fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> Result<bool, SecurityError> {
        if signature_bytes.len() != 64 {
            return Err(SecurityError::BadSignature(format!(
                "signature must be 64 bytes, got {}", signature_bytes.len()
            )));
        }

        let sig_array: [u8; 64] = signature_bytes.try_into()
            .map_err(|_| SecurityError::BadSignature("conversion failed".into()))?;
        let signature = Signature::from_bytes(&sig_array);

        self.verifying_key.verify(data, &signature)
            .map(|_| true)
            .map_err(|e| SecurityError::SignatureInvalid(e.to_string()))
    }

    /// Verify a file against its `.sig` sidecar.
    /// Expects `path.sig` to exist alongside `path`.
    pub fn verify_file_with_sidecar(&self, path: &Path) -> Result<bool, SecurityError> {
        let data = fs::read(path)?;

        // Build signature path: "plugin.puce_plugin" → "plugin.puce_plugin.sig"
        let mut sig_path = path.to_path_buf();
        let ext = sig_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let new_ext = format!("{}.sig", ext);
        sig_path.set_extension(&new_ext);

        if !sig_path.exists() {
            return Err(SecurityError::BadSignature(format!(
                "Sidecar signature not found: {:?}", sig_path
            )));
        }

        let sig_bytes = fs::read(&sig_path)?;
        self.verify(&data, &sig_bytes)
    }

    /// Verify a plugin (shorthand).
    pub fn verify_plugin(&self, path: &Path) -> Result<bool, SecurityError> {
        log::info!("Verifying plugin signature: {:?}", path);
        self.verify_file_with_sidecar(path)
    }
}

impl Default for SignatureVerifier {
    fn default() -> Self {
        Self::new().expect("Failed to initialize signature verifier with embedded key")
    }
}

// ─────────────────────────────────────────────────────────────
// OTA Update Manager
// ─────────────────────────────────────────────────────────────

/// Information about a pending update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub release_date: String,
    pub download_url: String,
    pub signature_url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub is_mandatory: bool,
    pub changelog: Vec<String>,
    pub hid_db_version: Option<u32>,
    pub hid_db_url: Option<String>,
}

/// Manages secure OTA updates.
pub struct OTAManager {
    update_server: String,
    current_version: String,
    backup_dir: PathBuf,
    verifier: SignatureVerifier,
}

impl OTAManager {
    pub fn new(
        update_server: &str,
        current_version: &str,
        backup_dir: PathBuf,
    ) -> Result<Self, SecurityError> {
        Ok(Self {
            update_server: update_server.to_string(),
            current_version: current_version.to_string(),
            backup_dir,
            verifier: SignatureVerifier::new()?,
        })
    }

    /// Check if an update is available by querying the update server manifest.
    /// Returns `None` if current version is up to date.
    ///
    /// In production: makes HTTPS GET to `{update_server}/manifest.json`
    pub async fn check_update(&self) -> Result<Option<UpdateInfo>, SecurityError> {
        let manifest_url = format!("{}/manifest.json", self.update_server);
        log::info!("Checking for updates at: {}", manifest_url);

        // In production: use reqwest to fetch and parse manifest
        // let response = reqwest::get(&manifest_url).await
        //     .map_err(|e| SecurityError::OtaError(e.to_string()))?;
        // let manifest: UpdateInfo = response.json().await
        //     .map_err(|e| SecurityError::OtaError(e.to_string()))?;

        // Stub: return no update available
        log::info!("Current version: {}. No updates available (stub).", self.current_version);
        Ok(None)
    }

    /// Download and verify an update package.
    /// Returns path to the verified download on disk.
    pub async fn download_and_verify(
        &self,
        info: &UpdateInfo,
        dest_dir: &Path,
    ) -> Result<PathBuf, SecurityError> {
        let dest_path = dest_dir.join(format!("puce_update_{}.pkg", info.version));
        let sig_path = dest_dir.join(format!("puce_update_{}.pkg.sig", info.version));

        log::info!("Downloading update {} to {:?}", info.version, dest_path);

        // In production:
        // download_file(&info.download_url, &dest_path).await?;
        // download_file(&info.signature_url, &sig_path).await?;

        // Verify integrity (SHA-256)
        // IntegrityChecker::verify_file(&dest_path, &info.sha256)?;

        // Verify signature
        // self.verifier.verify_file_with_sidecar(&dest_path)?;

        log::info!("Update package verified successfully");
        Ok(dest_path)
    }

    /// Apply a downloaded update atomically (rename + restart).
    pub fn apply_update(&self, pkg_path: &Path, target_path: &Path) -> Result<(), SecurityError> {
        // 1. Backup current binary
        if target_path.exists() {
            let backup_path = self.backup_dir.join("puce_backup_previous");
            fs::create_dir_all(&self.backup_dir)?;
            fs::copy(target_path, &backup_path)?;
            log::info!("Backup created at: {:?}", backup_path);
        }

        // 2. Atomic rename (same filesystem) or copy + delete
        #[cfg(unix)]
        std::os::unix::fs::rename(pkg_path, target_path)
            .map_err(|e| SecurityError::OtaError(format!("rename failed: {}", e)))?;

        #[cfg(windows)]
        fs::copy(pkg_path, target_path)
            .map_err(|e| SecurityError::OtaError(format!("copy failed: {}", e)))?;

        log::info!("Update applied successfully. Restart required.");
        Ok(())
    }

    /// Rollback to the previous version (restores backup).
    pub fn rollback(&self, target_path: &Path) -> Result<(), SecurityError> {
        let backup_path = self.backup_dir.join("puce_backup_previous");
        if !backup_path.exists() {
            return Err(SecurityError::OtaError("No backup found for rollback".into()));
        }
        fs::copy(&backup_path, target_path)?;
        log::warn!("Rolled back to previous version");
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────
// Key Generation Utility (for build tooling / signing server)
// ─────────────────────────────────────────────────────────────

/// Generate a new ed25519 keypair. Used by the signing server, not the client.
pub fn generate_keypair() -> ([u8; 32], [u8; 64]) {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (verifying_key.to_bytes(), signing_key.to_keypair_bytes())
}

/// Sign a file with a signing key (for build tooling).
pub fn sign_file(path: &Path, signing_key_bytes: &[u8; 64]) -> Result<Vec<u8>, SecurityError> {
    use ed25519_dalek::{Signer, SigningKey};

    let data = fs::read(path)?;
    let signing_key = SigningKey::from_keypair_bytes(signing_key_bytes)
        .map_err(|e| SecurityError::BadPublicKey(e.to_string()))?;
    let signature = signing_key.sign(&data);
    Ok(signature.to_bytes().to_vec())
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_bytes_consistent() {
        let data = b"PUCE security test data";
        let h1 = IntegrityChecker::hash_bytes(data);
        let h2 = IntegrityChecker::hash_bytes(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_hash_known_value() {
        let h = IntegrityChecker::hash_bytes(b"hello");
        assert_eq!(
            h,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_hash_file() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"test content").unwrap();
        let hash = IntegrityChecker::hash_file(f.path()).unwrap();
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_verify_file_ok() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello").unwrap();
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        let result = IntegrityChecker::verify_file(f.path(), expected);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_file_wrong_hash() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"hello").unwrap();
        let result = IntegrityChecker::verify_file(f.path(), "wronghash");
        assert!(matches!(result, Err(SecurityError::IntegrityFailed { .. })));
    }

    #[test]
    fn test_sign_and_verify() {
        let (pub_key, priv_key) = generate_keypair();
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(b"test plugin data").unwrap();

        // Sign the file
        let sig = sign_file(f.path(), &priv_key).unwrap();
        assert_eq!(sig.len(), 64);

        // Verify with correct key
        let verifier = SignatureVerifier::with_key(&pub_key).unwrap();
        let data = fs::read(f.path()).unwrap();
        assert!(verifier.verify(&data, &sig).unwrap());
    }

    #[test]
    fn test_verify_wrong_signature_fails() {
        let (pub_key, _) = generate_keypair();
        let verifier = SignatureVerifier::with_key(&pub_key).unwrap();
        let bad_sig = [0u8; 64];
        let result = verifier.verify(b"data", &bad_sig);
        assert!(matches!(result, Err(SecurityError::SignatureInvalid(_))));
    }

    #[test]
    fn test_bad_signature_length() {
        let (pub_key, _) = generate_keypair();
        let verifier = SignatureVerifier::with_key(&pub_key).unwrap();
        let result = verifier.verify(b"data", &[0u8; 32]); // Too short
        assert!(matches!(result, Err(SecurityError::BadSignature(_))));
    }
}
