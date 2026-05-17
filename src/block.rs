use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::{
    merkle::{compute_merkle_root, Hash, MerkleTree},
    transaction::Transaction,
};

#[derive(Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub merkle_root: Hash,
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
        let merkle_root = compute_merkle_root(&transactions);
        Block {
            index,
            timestamp,
            transactions,
            merkle_root,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        }
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string());
        hasher.update(self.timestamp.to_string());
        hasher.update(self.merkle_root);
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

    pub fn prove_transaction(&self, index: usize) -> Vec<Hash> {
        let leaves: Vec<Hash> = self.transactions.iter().map(Transaction::hash).collect();
        MerkleTree::from_leaves(leaves).prove(index)
    }
}
