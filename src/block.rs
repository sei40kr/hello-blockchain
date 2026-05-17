use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

#[derive(Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        for transaction in &self.transactions {
            hasher.update(transaction.sender.as_bytes());
            hasher.update(transaction.recipient.as_bytes());
            hasher.update(transaction.amount.to_le_bytes());
            hasher.update(transaction.signature.to_bytes());
        }
        hasher.update(self.previous_hash.clone());
        hasher.update(self.nonce.to_string());

        format!("{:x}", hasher.finalize())
    }

    pub fn mine(&mut self, difficulty: u32) {
        let target = "0".repeat(difficulty as usize);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}
