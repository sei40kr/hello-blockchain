use crate::{block::Block, transaction::Transaction};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
}

impl Blockchain {
    pub fn new(difficulty: u32) -> Self {
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

    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<(), &'static str> {
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

    pub fn is_valid(&self) -> bool {
        self.chain.iter().all(|block| {
            block.hash == block.calculate_hash()
                && block.transactions.iter().all(Transaction::is_valid)
        }) && self
            .chain
            .windows(2)
            .all(|pair| pair[0].hash == pair[1].previous_hash)
    }
}
