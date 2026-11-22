//! archlinux-keyring — pacman keyring CLI.

use sigmaos_archlinux_keyring::{Keyring, MasterKey, KeyTrust, build_keyring};
use sigmaos_archlinux_keyring::key::UserId;

fn main() {
    let mut kr = Keyring::new();
    kr.init();
    // Demo: populate with a sample Arch master key.
    let fp = "4fc152433b6a755b78a5b154f3a1d55d4f1e5a2b".to_string();
    let key = MasterKey {
        fingerprint: fp.clone(), created: 0, expires: 0,
        user_ids: vec![UserId::parse("Arch Linux Master Key <root@archlinux.org>")],
        trust: KeyTrust::Unknown, revoked: sigmaos_archlinux_keyring::Revocation::None,
        signing_subkeys: vec![], encryption_subkeys: vec![],
    };
    kr.add_key(key).unwrap();
    kr.trust(&fp, KeyTrust::Full).unwrap();
    let stats = kr.stats();
    println!("archlinux-keyring: {} keys ({} valid)", stats.total, stats.valid);
    let _ = build_keyring(vec![]);
}
