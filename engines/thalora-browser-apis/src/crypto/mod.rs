//! Web Crypto API
//!
//! This module implements the Web Crypto API, providing cryptographic functionality:
//! - `crypto.getRandomValues()` - Generate random values
//! - `crypto.randomUUID()` - Generate random UUIDs
//! - `crypto.subtle` - SubtleCrypto interface for advanced operations

pub mod crypto;
pub mod crypto_key;
pub mod subtle_crypto;

pub use crypto::Crypto;
pub use crypto_key::{Algorithm, CryptoKeyData, KeyMaterial, KeyType, KeyUsage};
pub use subtle_crypto::SubtleCrypto;

#[cfg(test)]
mod tests;
