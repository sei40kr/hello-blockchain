mod block;
mod blockchain;
mod merkle;
mod node;
mod transaction;
mod wallet;

use std::{cell::RefCell, rc::Rc};

use crate::{
    merkle::{sha256, MerkleTree},
    node::Node,
    wallet::Wallet,
};

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
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
    let transaction3 = charlie.sign(alice.verifying_key(), 25);

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
        .add_block(vec![
            transaction1.clone(),
            transaction2.clone(),
            transaction3.clone(),
        ])
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

    // Demonstrate Merkle inclusion proof for the 3-tx block (odd count triggers
    // Bitcoin-style duplication of the last leaf).
    let n3 = node3.borrow();
    let three_tx_block = n3
        .blockchain
        .chain
        .iter()
        .find(|b| b.transactions.len() == 3)
        .expect("node 3 should still hold the 3-tx block");
    let proof = three_tx_block.prove_transaction(1);
    let leaf = transaction2.hash();
    let ok = MerkleTree::verify(&three_tx_block.merkle_root, &leaf, 1, &proof);
    println!("Merkle root:               {}", hex(&three_tx_block.merkle_root));
    println!("Proof for transaction[1]:  {} siblings", proof.len());
    println!("Inclusion proof verifies:  {}", ok);
    assert!(ok);

    let bad_leaf = sha256(b"not in the block");
    assert!(!MerkleTree::verify(
        &three_tx_block.merkle_root,
        &bad_leaf,
        1,
        &proof,
    ));
}
