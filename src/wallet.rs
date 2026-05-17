use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

pub struct Wallet {
    signing_key: SigningKey,
}

impl Wallet {
    const ADDRESS_VERSION: u8 = 0x00;

    pub fn new() -> Self {
        Wallet {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub fn address(&self) -> String {
        let sha = Sha256::digest(self.verifying_key().as_bytes());
        let ripemd = Ripemd160::digest(sha);
        bs58::encode(ripemd)
            .with_check_version(Self::ADDRESS_VERSION)
            .into_string()
    }

    pub fn sign(&self, recipient: VerifyingKey, amount: u64) -> Transaction {
        Transaction::new(&self.signing_key, recipient, amount)
    }
}
