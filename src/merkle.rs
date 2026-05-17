use sha2::{Digest, Sha256};

use crate::transaction::Transaction;

pub type Hash = [u8; 32];

pub const EMPTY_MERKLE_ROOT: Hash = [0u8; 32];

pub fn sha256(bytes: &[u8]) -> Hash {
    Sha256::digest(bytes).into()
}

pub fn hash_pair(left: &Hash, right: &Hash) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

pub struct MerkleTree {
    levels: Vec<Vec<Hash>>,
}

impl MerkleTree {
    pub fn from_leaves(leaves: Vec<Hash>) -> Self {
        if leaves.is_empty() {
            return MerkleTree {
                levels: vec![vec![EMPTY_MERKLE_ROOT]],
            };
        }
        let mut levels = vec![leaves];
        while levels.last().unwrap().len() > 1 {
            let current = levels.last().unwrap();
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            for chunk in current.chunks(2) {
                let left = &chunk[0];
                // Bitcoin-style: duplicate the last leaf when the count is odd.
                let right = if chunk.len() == 2 { &chunk[1] } else { &chunk[0] };
                next.push(hash_pair(left, right));
            }
            levels.push(next);
        }
        MerkleTree { levels }
    }

    pub fn root(&self) -> Hash {
        *self.levels.last().unwrap().first().unwrap()
    }

    pub fn prove(&self, mut index: usize) -> Vec<Hash> {
        let leaf_count = self.levels[0].len();
        assert!(index < leaf_count, "leaf index out of range");

        let mut proof = Vec::with_capacity(self.levels.len().saturating_sub(1));
        for level in &self.levels[..self.levels.len() - 1] {
            let sibling = if index.is_multiple_of(2) {
                // Right sibling; duplicate self if absent (odd tail).
                if index + 1 < level.len() {
                    level[index + 1]
                } else {
                    level[index]
                }
            } else {
                level[index - 1]
            };
            proof.push(sibling);
            index /= 2;
        }
        proof
    }

    pub fn verify(root: &Hash, leaf: &Hash, mut index: usize, proof: &[Hash]) -> bool {
        let mut current = *leaf;
        for sibling in proof {
            current = if index.is_multiple_of(2) {
                hash_pair(&current, sibling)
            } else {
                hash_pair(sibling, &current)
            };
            index /= 2;
        }
        current == *root
    }
}

pub fn compute_merkle_root(transactions: &[Transaction]) -> Hash {
    let leaves: Vec<Hash> = transactions.iter().map(Transaction::hash).collect();
    MerkleTree::from_leaves(leaves).root()
}
