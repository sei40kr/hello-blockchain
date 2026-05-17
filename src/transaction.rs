use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

#[derive(Clone)]
pub struct Transaction {
    pub sender: VerifyingKey,
    pub recipient: VerifyingKey,
    pub amount: u64,
    pub signature: Signature,
}

impl Transaction {
    pub fn new(sender_key: &SigningKey, recipient: VerifyingKey, amount: u64) -> Self {
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

    pub fn is_valid(&self) -> bool {
        let payload = Self::signing_payload(&self.sender, &self.recipient, self.amount);
        self.sender.verify(&payload, &self.signature).is_ok()
    }
}
