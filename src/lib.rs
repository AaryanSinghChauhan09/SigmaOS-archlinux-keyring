//! SigmaOS archlinux-keyring component.
//!
//! Rust implementation of `archlinux/archlinux-keyring`: the Arch Linux
//! pacman keyring — trusted package-signing keys packaged for pacman.
//!
//! This port implements the keyring domain model: trusted master keys
//! (with fingerprints/expiry), pacman-key semantics (init, populate,
//! refresh-keys, trust/untrust), and keyring manifest generation —
//! the logic upstream's PKGBUILD/scripts produce.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod key;
pub mod keyring;

pub use error::KeyringError;
pub use key::{KeyTrust, MasterKey, Revocation, UserId};
pub use keyring::{Keyring, PopulatePlan, build_keyring, verify_signature_meta};
