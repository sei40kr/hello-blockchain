use std::{cell::RefCell, rc::Rc};

use crate::{blockchain::Blockchain, transaction::Transaction};

pub struct Node {
    pub blockchain: Blockchain,
    other_nodes: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(difficulty: u32) -> Self {
        Node {
            blockchain: Blockchain::new(difficulty),
            other_nodes: vec![],
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), &'static str> {
        self.blockchain.add_block(transactions)
    }

    pub fn is_valid(&self) -> bool {
        self.blockchain.is_valid()
    }

    pub fn consensus(&mut self) {
        let mut longest_chain = self.blockchain.chain.clone();

        for node in &self.other_nodes {
            let node_ref = node.borrow();
            if node_ref.is_valid() && node_ref.blockchain.chain.len() > longest_chain.len() {
                longest_chain = node_ref.blockchain.chain.clone();
            }
        }

        self.blockchain.chain = longest_chain;
    }

    pub fn add_other_node(&mut self, node: Rc<RefCell<Node>>) {
        self.other_nodes.push(node);
    }
}
