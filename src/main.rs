use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

struct Wallet {
    signing_key: SigningKey,
}

impl Wallet {
    const ADDRESS_VERSION: u8 = 0x00;

    fn new() -> Self {
        Wallet {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    fn address(&self) -> String {
        let sha = Sha256::digest(self.verifying_key().as_bytes());
        let ripemd = Ripemd160::digest(sha);
        bs58::encode(ripemd)
            .with_check_version(Self::ADDRESS_VERSION)
            .into_string()
    }

    fn sign(&self, recipient: VerifyingKey, amount: u64) -> Transaction {
        Transaction::new(&self.signing_key, recipient, amount)
    }
}

#[derive(Clone)]
struct Transaction {
    sender: VerifyingKey,
    recipient: VerifyingKey,
    amount: u64,
    signature: Signature,
}

impl Transaction {
    fn new(sender_key: &SigningKey, recipient: VerifyingKey, amount: u64) -> Self {
        let sender = sender_key.verifying_key();
        let signature = sender_key.sign(&Self::signing_payload(&sender, &recipient, amount));
        Transaction {
            sender,
            recipient,
            amount,
            signature,
        }
    }

    fn signing_payload(sender: &VerifyingKey, recipient: &VerifyingKey, amount: u64) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + 32 + 8);
        bytes.extend_from_slice(sender.as_bytes());
        bytes.extend_from_slice(recipient.as_bytes());
        bytes.extend_from_slice(&amount.to_le_bytes());
        bytes
    }

    fn is_valid(&self) -> bool {
        let payload = Self::signing_payload(&self.sender, &self.recipient, self.amount);
        self.sender.verify(&payload, &self.signature).is_ok()
    }
}

#[derive(Clone)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {
    fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
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

    fn calculate_hash(&self) -> String {
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

    fn mine(&mut self, difficulty: u32) {
        let target = "0".repeat(difficulty as usize);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
    }
}

struct Blockchain {
    chain: Vec<Block>,
    difficulty: u32,
}

impl Blockchain {
    fn new(difficulty: u32) -> Self {
        Blockchain {
            chain: vec![Blockchain::create_genesis_block()],
            difficulty,
        }
    }

    fn create_genesis_block() -> Block {
        let mut block = Block::new(0, vec![], "0".to_string());
        block.mine(1);
        block
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), &'static str> {
        if !transactions.iter().all(Transaction::is_valid) {
            return Err("block contains a transaction with an invalid signature");
        }
        let previous_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            transactions,
            previous_block.hash.clone(),
        );
        new_block.mine(self.difficulty);
        self.chain.push(new_block);
        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.chain.iter().all(|block| {
            block.hash == block.calculate_hash()
                && block.transactions.iter().all(Transaction::is_valid)
        }) && self
            .chain
            .windows(2)
            .all(|pair| pair[0].hash == pair[1].previous_hash)
    }
}

struct Node {
    blockchain: Blockchain,
    other_nodes: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    fn new(difficulty: u32) -> Self {
        Node {
            blockchain: Blockchain::new(difficulty),
            other_nodes: vec![],
        }
    }

    fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), &'static str> {
        self.blockchain.add_block(transactions)
    }

    fn is_valid(&self) -> bool {
        self.blockchain.is_valid()
    }

    fn consensus(&mut self) {
        let mut longest_chain = self.blockchain.chain.clone();

        for node in &self.other_nodes {
            let node_ref = node.borrow();
            if node_ref.is_valid() && node_ref.blockchain.chain.len() > longest_chain.len() {
                longest_chain = node_ref.blockchain.chain.clone();
            }
        }

        self.blockchain.chain = longest_chain;
    }

    fn add_other_node(&mut self, node: Rc<RefCell<Node>>) {
        self.other_nodes.push(node);
    }
}

fn main() {
    let difficulty = 4;
    let node1 = Rc::new(RefCell::new(Node::new(difficulty)));
    let node2 = Rc::new(RefCell::new(Node::new(difficulty)));
    let node3 = Rc::new(RefCell::new(Node::new(difficulty)));

    node1.borrow_mut().add_other_node(Rc::clone(&node2));
    node1.borrow_mut().add_other_node(Rc::clone(&node3));
    node2.borrow_mut().add_other_node(Rc::clone(&node1));
    node2.borrow_mut().add_other_node(Rc::clone(&node3));
    node3.borrow_mut().add_other_node(Rc::clone(&node1));
    node3.borrow_mut().add_other_node(Rc::clone(&node2));

    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    println!("Alice's address:   {}", alice.address());
    println!("Bob's address:     {}", bob.address());
    println!("Charlie's address: {}", charlie.address());

    let transaction1 = alice.sign(bob.verifying_key(), 100);
    let transaction2 = bob.sign(charlie.verifying_key(), 50);

    node1
        .borrow_mut()
        .add_block(vec![transaction1.clone()])
        .unwrap();
    node2
        .borrow_mut()
        .add_block(vec![transaction1.clone()])
        .unwrap();
    node3
        .borrow_mut()
        .add_block(vec![transaction1.clone(), transaction2.clone()])
        .unwrap();

    let mut tampered = transaction1.clone();
    tampered.amount = 999_999;
    assert!(
        node1.borrow_mut().add_block(vec![tampered]).is_err(),
        "tampered transaction must be rejected"
    );

    node1.borrow_mut().consensus();
    node2.borrow_mut().consensus();
    node3.borrow_mut().consensus();

    println!(
        "Node 1 blockchain length: {}",
        node1.borrow().blockchain.chain.len(),
    );
    println!(
        "Node 2 blockchain length: {}",
        node2.borrow().blockchain.chain.len(),
    );
    println!(
        "Node 3 blockchain length: {}",
        node3.borrow().blockchain.chain.len(),
    );
}
