//! OpenPGP key model — master keys, subkeys, user IDs.

/// Trust level (GnuPG ownertrust).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyTrust {
    /// Unknown trust.
    Unknown,
    /// Never trusted.
    Never,
    /// Marginally trusted.
    Marginal,
    /// Fully trusted.
    Full,
    /// Ultimately trusted.
    Ultimate,
}

/// A user ID on a key.
#[derive(Debug, Clone)]
pub struct UserId {
    /// Name.
    pub name: String,
    /// Email.
    pub email: String,
    /// Comment.
    pub comment: String,
}

impl UserId {
    /// Parse `Name (comment) <email>` format.
    pub fn parse(text: &str) -> Self {
        let email = text
            .rsplit('<')
            .next()
            .and_then(|s| s.strip_suffix('>'))
            .unwrap_or("")
            .trim()
            .to_string();
        let before = text.split('<').next().unwrap_or("").trim();
        let (name, comment) = match before.split_once('(') {
            Some((n, c)) => (n.trim().to_string(), c.trim_end_matches(')').trim().to_string()),
            None => (before.to_string(), String::new()),
        };
        Self { name, email, comment }
    }

    /// Display form.
    pub fn display(&self) -> String {
        if self.comment.is_empty() {
            format!("{} <{}>", self.name, self.email)
        } else {
            format!("{} ({}) <{}>", self.name, self.comment, self.email)
        }
    }
}

/// Key revocation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revocation {
    /// Not revoked.
    None,
    /// Soft-revoked (key still usable but flagged).
    Soft,
    /// Hard-revoked (key must not be used).
    Hard,
}

/// A master key in the keyring.
#[derive(Debug, Clone)]
pub struct MasterKey {
    /// Primary fingerprint (40 hex chars, no spaces).
    pub fingerprint: String,
    /// Key creation timestamp (unix).
    pub created: i64,
    /// Expiration timestamp (unix, 0 = never).
    pub expires: i64,
    /// User IDs.
    pub user_ids: Vec<UserId>,
    /// Trust level.
    pub trust: KeyTrust,
    /// Revocation status.
    pub revoked: Revocation,
    /// Signing subkey fingerprints.
    pub signing_subkeys: Vec<String>,
    /// Encryption subkey fingerprints.
    pub encryption_subkeys: Vec<String>,
}

impl MasterKey {
    /// Whether the key is currently valid (not expired, not hard-revoked).
    pub fn valid_at(&self, now: i64) -> bool {
        self.revoked != Revocation::Hard
            && (self.expires == 0 || now < self.expires)
    }

    /// Primary user ID display.
    pub fn primary_uid(&self) -> Option<&UserId> {
        self.user_ids.first()
    }

    /// Whether fingerprint format is valid (40 hex chars).
    pub fn valid_fingerprint(fp: &str) -> bool {
        fp.len() == 40 && fp.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uid_parse() {
        let u = UserId::parse("Arch Linux Release Engineering (arch-release) <arch-release@archlinux.org>");
        assert_eq!(u.name, "Arch Linux Release Engineering");
        assert_eq!(u.comment, "arch-release");
        assert_eq!(u.email, "arch-release@archlinux.org");
    }

    #[test]
    fn test_key_validity() {
        let k = MasterKey {
            fingerprint: "a".repeat(40),
            created: 0, expires: 100,
            user_ids: vec![], trust: KeyTrust::Full,
            revoked: Revocation::None,
            signing_subkeys: vec![], encryption_subkeys: vec![],
        };
        assert!(k.valid_at(50));
        assert!(!k.valid_at(150));
    }

    #[test]
    fn test_fingerprint() {
        assert!(MasterKey::valid_fingerprint(&"a".repeat(40)));
        assert!(!MasterKey::valid_fingerprint("abc"));
        assert!(!MasterKey::valid_fingerprint(&"g".repeat(40)));
    }
}
