//! Keyring model — pacman-key init/populate/refresh semantics.

use std::collections::BTreeMap;

use crate::key::{KeyTrust, MasterKey, Revocation};
use crate::KeyringError;

/// The pacman keyring.
#[derive(Debug, Default)]
pub struct Keyring {
    /// Trusted master keys by fingerprint.
    pub masters: BTreeMap<String, MasterKey>,
    /// Whether pacman-key --init has been run (GNUPGHOME exists).
    pub initialized: bool,
}

impl Keyring {
    /// New empty keyring (pacman-key --init state).
    pub fn new() -> Self {
        Self { masters: BTreeMap::new(), initialized: false }
    }

    /// pacman-key --init: create keyring structure.
    pub fn init(&mut self) {
        self.initialized = true;
        self.masters.clear();
    }

    /// Add a master key (pacman-key --add / populate).
    pub fn add_key(&mut self, key: MasterKey) -> Result<(), KeyringError> {
        if !MasterKey::valid_fingerprint(&key.fingerprint) {
            return Err(KeyringError::InvalidFingerprint(key.fingerprint.clone()));
        }
        self.masters.insert(key.fingerprint.clone(), key);
        Ok(())
    }

    /// Trust a key (pacman-key --lsign / ownertrust).
    pub fn trust(&mut self, fingerprint: &str, level: KeyTrust) -> Result<(), KeyringError> {
        let k = self
            .masters
            .get_mut(fingerprint)
            .ok_or_else(|| KeyringError::NotFound(fingerprint.to_string()))?;
        k.trust = level;
        Ok(())
    }

    /// Untrust a key.
    pub fn untrust(&mut self, fingerprint: &str) -> Result<(), KeyringError> {
        self.trust(fingerprint, KeyTrust::Never)
    }

    /// Revoke a key.
    pub fn revoke(&mut self, fingerprint: &str, hard: bool) -> Result<(), KeyringError> {
        let k = self
            .masters
            .get_mut(fingerprint)
            .ok_or_else(|| KeyringError::NotFound(fingerprint.to_string()))?;
        k.revoked = if hard { Revocation::Hard } else { Revocation::Soft };
        Ok(())
    }

    /// Valid keys at time `now` (not expired, not hard-revoked, trusted).
    pub fn valid_keys(&self, now: i64) -> Vec<&MasterKey> {
        self.masters
            .values()
            .filter(|k| k.valid_at(now) && k.trust >= KeyTrust::Marginal)
            .collect()
    }

    /// Keys needing refresh (expires within `within_secs`).
    pub fn keys_needing_refresh(&self, now: i64, within_secs: i64) -> Vec<&MasterKey> {
        self.masters
            .values()
            .filter(|k| {
                k.expires > 0 && k.expires > now && k.expires - now < within_secs
            })
            .collect()
    }

    /// Keyring statistics.
    pub fn stats(&self) -> KeyringStats {
        let total = self.masters.len();
        let valid = self.masters.values().filter(|k| k.revoked == Revocation::None).count();
        let expired = self.masters.values().filter(|k| k.expires > 0 && k.expires < 0).count();
        KeyringStats { total, valid, expired }
    }
}

/// Keyring stats summary.
#[derive(Debug, Default)]
pub struct KeyringStats {
    /// Total master keys.
    pub total: usize,
    /// Non-revoked keys.
    pub valid: usize,
    /// Expired keys.
    pub expired: usize,
}

/// Plan for populating a fresh keyring (pacman-key --populate).
#[derive(Debug)]
pub struct PopulatePlan {
    /// Keys to import (archlinux-keyring package contents).
    pub keys_to_add: Vec<MasterKey>,
    /// Keys to locally-sign (trusted).
    pub keys_to_trust: Vec<String>,
}

/// Build populate plan: import all keys, trust the Arch master keys.
pub fn build_keyring(master_keys: Vec<MasterKey>) -> PopulatePlan {
    let trust_fps: Vec<String> = master_keys.iter().map(|k| k.fingerprint.clone()).collect();
    PopulatePlan {
        keys_to_add: master_keys,
        keys_to_trust: trust_fps,
    }
}

/// Verify signature metadata: check a signing key fingerprint is trusted.
pub fn verify_signature_meta(
    keyring: &Keyring,
    signer_fingerprint: &str,
    sig_timestamp: i64,
) -> Result<(), KeyringError> {
    let key = keyring
        .masters
        .get(signer_fingerprint)
        .ok_or_else(|| KeyringError::NotFound(signer_fingerprint.to_string()))?;
    if !key.valid_at(sig_timestamp) {
        return Err(KeyringError::InvalidFingerprint(format!(
            "key expired or revoked at signature time"
        )));
    }
    if key.trust < KeyTrust::Marginal {
        return Err(KeyringError::InvalidFingerprint("key not trusted".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::UserId;

    fn key(fp: &str, expires: i64) -> MasterKey {
        MasterKey {
            fingerprint: fp.to_string(),
            created: 0,
            expires,
            user_ids: vec![UserId::parse("Test <t@t>")],
            trust: KeyTrust::Unknown,
            revoked: Revocation::None,
            signing_subkeys: vec![],
            encryption_subkeys: vec![],
        }
    }

    #[test]
    fn test_init_populate() {
        let mut kr = Keyring::new();
        assert!(!kr.initialized);
        kr.init();
        assert!(kr.initialized);
        kr.add_key(key(&"a".repeat(40), 0)).unwrap();
        assert_eq!(kr.masters.len(), 1);
    }

    #[test]
    fn test_trust_and_verify() {
        let mut kr = Keyring::new();
        kr.init();
        let fp = "b".repeat(40);
        kr.add_key(key(&fp, 0)).unwrap();
        kr.trust(&fp, KeyTrust::Full).unwrap();
        assert!(verify_signature_meta(&kr, &fp, 100).is_ok());
        // Untrusted key fails.
        let other = "c".repeat(40);
        kr.add_key(key(&other, 0)).unwrap();
        assert!(verify_signature_meta(&kr, &other, 100).is_err());
    }

    #[test]
    fn test_refresh_detection() {
        let mut kr = Keyring::new();
        kr.init();
        kr.add_key(key(&"d".repeat(40), 50_000)).unwrap(); // expires soon
        kr.add_key(key(&"e".repeat(40), 0)).unwrap(); // never expires
        let refresh = kr.keys_needing_refresh(0, 100_000);
        assert_eq!(refresh.len(), 1);
    }
}
