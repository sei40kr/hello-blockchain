mod block;
mod blockchain;
mod node;
mod transaction;
mod wallet;

use std::{cell::RefCell, rc::Rc};

use crate::{node::Node, wallet::Wallet};

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
