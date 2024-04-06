use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};

#[derive(Clone)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: u64,
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
            hasher.update(&transaction.sender.clone());
            hasher.update(&transaction.recipient.clone());
            hasher.update(&transaction.amount.to_string());
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

    fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous_block = self.chain.last().unwrap();
        let mut new_block = Block::new(
            previous_block.index + 1,
            transactions,
            previous_block.hash.clone(),
        );
        new_block.mine(self.difficulty);
        self.chain.push(new_block);
    }

    fn is_valid(&self) -> bool {
        self.chain
            .iter()
            .all(|block| block.hash == block.calculate_hash())
            && self
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

    fn add_block(&mut self, transactions: Vec<Transaction>) {
        self.blockchain.add_block(transactions);
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

    let transaction1 = Transaction {
        sender: String::from("Alice"),
        recipient: String::from("Bob"),
        amount: 100,
    };

    let transaction2 = Transaction {
        sender: String::from("Bob"),
        recipient: String::from("Charlie"),
        amount: 50,
    };

    node1.borrow_mut().add_block(vec![transaction1.clone()]);
    node2.borrow_mut().add_block(vec![transaction1.clone()]);
    node3
        .borrow_mut()
        .add_block(vec![transaction1.clone(), transaction2.clone()]);

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