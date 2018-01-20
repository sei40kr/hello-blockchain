# coding: utf-8

import hashlib
import json
from time import time


class BlockChain(object):
    @staticmethod
    def hash(block):
        '''hash
        :param block: <dict> block
        :return <str>
        '''

        block_string = json.dumps(block, sort_keys=True).encode()
        return hashlib.sha256(block_string).hexdigest()

    def __init__(self):
        self.chain = []
        self.current_transactions = []

        # create a genesis block
        self.new_block(previous_hash=1, proof=100)

    @property
    def last_block(self):
        return self.chain[-1]

    def new_block(self, proof, previous_hash=None):
        '''new_block
        :param proof:
        :param previous_hash:
        '''

        block = {
            'index': len(self.chain) + 1,
            'timestamp': time(),
            'transactions': self.current_transactions,
            'proof': proof,
            'previous_hash': previous_hash or self.hash(self.chain[-1]),
        }

    def new_transaction(self, sender, recipient, amount):
        '''new_transaction
        :param sender: <str> sender's address
        :param recipient: <str> recipient's address
        :param amount: <int> amount
        :return <int> block address where this transaction will be included
        '''

        self.current_transactions.append({
            'sender': sender,
            'recipient': recipient,
            'amount': amount,
        })

        return self.last_block['index'] + 1
