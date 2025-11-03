use rand::rngs::OsRng;
use k256::{ecdsa::{SigningKey, signature::DigestSigner}, elliptic_curve::sec1::ToEncodedPoint, FieldBytes};
use hex;
use sha2::Digest;

use crate::transaction::{Transaction,transaction_message};


pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
    signing_key: SigningKey,
}


impl Wallet {
    pub fn new() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        let verify_key = signing_key.verifying_key();

        // Clave privada en hex
        let private_key_bytes = signing_key.to_bytes();
        let private_key_hex = hex::encode(private_key_bytes);

        // Clave pública comprimida (33 bytes) en hex
        let public_point = verify_key.to_encoded_point(true);
        let public_key_hex = hex::encode(public_point.as_bytes());

        // Dirección simple = hash sha256(pubkey) y tomar 20 bytes (pseudo, no mainnet)
        let digest = sha2::Sha256::digest(public_point.as_bytes());
        let address = hex::encode(&digest[..20]);

        Wallet {
            private_key: private_key_hex,
            public_key: public_key_hex,
            address,
            signing_key,
        }
    }

    pub fn create_signed_transaction(&self, recipient: String, amount: u128, nonce: u64, chain_id: u32) -> Transaction {
        // firmar el mensaje canónico con la clave privada interna
        let signing_key = &self.signing_key;

        let msg = transaction_message(&self.address, &recipient, amount, nonce, chain_id);
        let mut hasher = sha2::Sha256::new();
        hasher.update(&msg);
        let signature: k256::ecdsa::Signature = signing_key.sign_digest(hasher);

        // clave pública comprimida en hex
        let verify_key = signing_key.verifying_key();
        let public_point = verify_key.to_encoded_point(true);
        let public_key_hex = hex::encode(public_point.as_bytes());

        Transaction {
            sender: self.address.clone(),
            recipient,
            amount,
            nonce,
            chain_id,
            public_key: Some(public_key_hex),
            signature: Some(hex::encode(signature.to_der().as_bytes())),
        }
    }

    // Reconstruye una Wallet desde una clave privada en hex (deriva pubkey y address)
    pub fn from_private_key_hex(private_key_hex: &str) -> Result<Self, String> {
        let sk_bytes = hex::decode(private_key_hex).map_err(|_| "clave privada hex inválida")?;
        let fb: FieldBytes = FieldBytes::clone_from_slice(&sk_bytes);
        let signing_key = SigningKey::from_bytes(&fb).map_err(|_| "clave privada inválida")?;

        let verify_key = signing_key.verifying_key();
        let public_point = verify_key.to_encoded_point(true);
        let public_key_hex = hex::encode(public_point.as_bytes());

        let digest = sha2::Sha256::digest(public_point.as_bytes());
        let address = hex::encode(&digest[..20]);

        Ok(Wallet {
            private_key: private_key_hex.to_string(),
            public_key: public_key_hex,
            address,
            signing_key,
        })
    }

    // Firma una transacción sin instancia previa, solo con la clave privada hex
    pub fn sign_transaction_from_private_key(
        private_key_hex: &str,
        recipient: String,
        amount: u128,
        nonce: u64,
        chain_id: u32,
    ) -> Result<Transaction, String> {
        let wallet = Wallet::from_private_key_hex(private_key_hex)?;
        let tx = wallet.create_signed_transaction(recipient, amount, nonce, chain_id);
        Ok(tx)
    }
}