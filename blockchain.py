# coding: utf-8

import hashlib
import json
from time import time
from urllib.parse import urlparse
from uuid import uuid4

import requests
from flask import Flask, jsonify, request


class Blockchain(object):
    @staticmethod
    def hash(block: dict) -> str:
        block_string = json.dumps(block, sort_keys=True).encode()
        return hashlib.sha256(block_string).hexdigest()

    @staticmethod
    def valid_proof(last_proof: int, proof: int) -> bool:
        guess = f'{last_proof}{proof}'.encode()
        guess_hash = hashlib.sha256(guess).hexdigest()

        return guess_hash[:4] == '0000'

    def __init__(self):
        self.chain = []
        self.current_transactions = []

        # Create node list as a collection set, to avoid duplicated nodes
        self.nodes = set()

        # Create a genesis block
        self.new_block(previous_hash=1, proof=100)

    @property
    def last_block(self) -> dict:
        return self.chain[-1]

    def register_node(self, url):
        """
        Add a new node to blockchain network.
        :param url: <str> Node address
        """
        urlobject = urlparse(url)
        self.nodes.add(urlobject.netloc)

    def new_block(self, proof: int, previous_hash: str = None) -> dict:
        block = {
            'index': len(self.chain) + 1,
            'timestamp': time(),
            'transactions': self.current_transactions,
            'proof': proof,
            'previous_hash': previous_hash or self.hash(self.chain[-1]),
        }

        self.current_transactions = []

        self.chain.append(block)
        return block

    def new_transaction(self, sender: str, recipient: str, amount: int) -> int:
        self.current_transactions.append({
            'sender': sender,
            'recipient': recipient,
            'amount': amount,
        })

        return self.last_block['index'] + 1

    def proof_of_work(self, last_proof: int) -> int:
        proof = 0
        while not self.valid_proof(last_proof, proof):
            proof += 1

        return proof

    def valid_chain(self, chain):
        """
        Check if specified chain is valid to the blockchain.

        :param chain: <list> A chain list
        :return <bool> Return true if specified one is valid
        """
        last_block = chain[0]

        for i, block in enumerate(chain):
            print(f'{last_block}')
            print(f'{block}')
            print(f'\n------------\n')

            # Check if the block in specified chain has valid hash
            if block['previous_hash'] != self.hash(last_block):
                return False

            # Check if the block in specified chain has valid proof
            if not self.valid_proof(last_block['proof'], block['proof']):
                return False

            last_block = block

        return True

    def resolve_conflicts(self):
        nodes = self.nodes

        # Find the longest chain
        max_len = len(self.chain)
        new_chain = None

        for node in nodes:
            response = requests.get('http://{node}/chain')

            if response.status_code != 200:
                continue

            body = response.json()
            length = body['length']
            chain = body['chain']

            if max_len < length and self.valid_chain(chain):
                max_len = length
                new_chain = chain

        if new_chain:
            self.chain = new_chain
            return True

        return False


node_address = str(uuid4()).replace('-', '')
instance = Blockchain()

app = Flask(__name__)


@app.route('/transaction/new', methods=['POST'])
def new_transaction():
    payload = request.get_json()

    keys = ['sender', 'recipient', 'amount']
    if not all(key in payload for key in keys):
        return 'Invalid request parameters.', 400

    index = instance.new_transaction(payload['sender'], payload['recipient'],
                                     payload['amount'])

    return jsonify({
        'message':
        f'Your transaction was successfully added to block {index}.'
    }), 201


@app.route('/mine', methods=['GET'])
def mine():
    proof = instance.proof_of_work(instance.last_block['proof'])
    block = instance.new_block(proof)

    # Node client(who is running this blockchain) can receive fees.
    instance.new_transaction(
        sender="0",
        recipient=node_address,
        amount=1,
    )

    return jsonify({
        'message': 'New block was found.',
        'index': block['index'],
        'transactions': block['transactions'],
        'proof': block['proof'],
        'previous_hash': block['previous_hash'],
    }), 200


@app.route('/chain', methods=['GET'])
def show_chain():
    return jsonify({
        'chain': instance.chain,
        'length': len(instance.chain),
    }), 200


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)


@app.route('/nodes/register', methods=['POST'])
def register_node():
    payload = request.get_json()
    nodes = payload.get('nodes')

    if nodes is None:
        return "Please supply a valid list of nodes", 400

    for node in nodes:
        instance.register_node(node)

    return jsonify({
        'message': 'New nodes have been added',
        'total_nodes': list(instance.nodes),
    }), 201


@app.route('/nodes/resolve', methods=['GET'])
def consensus():
    replaced = instance.resolve_conflicts()

    if replaced:
        response = {'message': 'Our chain was replaced',
                    'new_chain': instance.chain}
    else:
        response = {'message': 'Our chain was authoritative',
                    'chain': instance.chain}

    return jsonify(response), 200
