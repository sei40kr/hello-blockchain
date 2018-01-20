# coding: utf-8

import hashlib
import json
from time import time


class BlockChain(object):
    @staticmethod
    def hash(block: dict) -> str:
        '''hash
        :param block: block
        '''

        block_string = json.dumps(block, sort_keys=True).encode()
        return hashlib.sha256(block_string).hexdigest()

    def __init__(self):
        self.chain = []
        self.current_transactions = []

        # create a genesis block
        self.new_block(previous_hash=1, proof=100)

    @property
    def last_block(self) -> dict:
        return self.chain[-1]

    def new_block(self, proof: int, previous_hash: str = None) -> dict:
        '''new_block
        :param proof: proof
        :param previous_hash: hash of previous block
        '''

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
        '''new_transaction
        :param sender: sender's address
        :param recipient: recipient's address
        :param amount: amount
        :return block address where this transaction will be included
        '''

        self.current_transactions.append({
            'sender': sender,
            'recipient': recipient,
            'amount': amount,
        })

        return self.last_block['index'] + 1
