//! Integration tests for sigmaos-archlinux-keyring.

use sigmaos_archlinux_keyring::*;
use sigmaos_archlinux_keyring::key::UserId;

fn key(fp: &str) -> MasterKey {
    MasterKey {
        fingerprint: fp.to_string(), created: 0, expires: 0,
        user_ids: vec![UserId::parse("Arch <a@a>")],
        trust: KeyTrust::Unknown, revoked: Revocation::None,
        signing_subkeys: vec![], encryption_subkeys: vec![],
    }
}

#[test]
fn test_full_keyring_workflow() {
    let mut kr = Keyring::new();
    kr.init();
    let fp = "f".repeat(40);
    kr.add_key(key(&fp)).unwrap();
    kr.trust(&fp, KeyTrust::Full).unwrap();
    assert!(verify_signature_meta(&kr, &fp, 1000).is_ok());
    let s = kr.stats();
    assert_eq!(s.total, 1);
}

#[test]
fn test_revocation_blocks_verify() {
    let mut kr = Keyring::new();
    kr.init();
    let fp = "e".repeat(40);
    kr.add_key(key(&fp)).unwrap();
    kr.trust(&fp, KeyTrust::Full).unwrap();
    kr.revoke(&fp, true).unwrap();
    assert!(verify_signature_meta(&kr, &fp, 1000).is_err());
}

#[test]
fn test_populate_plan() {
    let keys = vec![key(&"a".repeat(40)), key(&"b".repeat(40))];
    let plan = build_keyring(keys);
    assert_eq!(plan.keys_to_add.len(), 2);
    assert_eq!(plan.keys_to_trust.len(), 2);
}
