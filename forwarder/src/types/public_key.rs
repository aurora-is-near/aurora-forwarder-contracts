use core::str::FromStr;

use crate::error::ContractError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicKey {
    /// ed25519 public keys are 32 bytes
    Ed25519([u8; 33]),
    /// secp256k1 keys are in the uncompressed 64 byte format
    Secp256k1([u8; 65]),
}

impl PublicKey {
    pub const fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Ed25519(bytes) => bytes,
            Self::Secp256k1(bytes) => bytes,
        }
    }
}

impl FromStr for PublicKey {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (key_type, key_data) = split_key_type_data(value)?;
        Ok(match key_type {
            KeyType::Ed25519 => {
                let mut buf = [0; 33];
                bs58::decode(key_data)
                    .onto(&mut buf[1..])
                    .expect("TODO: panic message");
                Self::Ed25519(buf)
            }
            KeyType::Secp256k1 => {
                let mut buf = [0; 65];
                buf[0] = 0x01;
                bs58::decode(key_data)
                    .onto(&mut buf[1..])
                    .expect("TODO: panic message");
                Self::Secp256k1(buf)
            }
        })
    }
}

fn split_key_type_data(value: &str) -> Result<(KeyType, &str), ContractError> {
    if let Some(idx) = value.find(':') {
        let (prefix, key_data) = value.split_at(idx);
        Ok((KeyType::from_str(prefix)?, &key_data[1..]))
    } else {
        // If there is no prefix then we Default to ED25519.
        Ok((KeyType::Ed25519, value))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum KeyType {
    Ed25519 = 0,
    Secp256k1 = 1,
}

impl FromStr for KeyType {
    type Err = ContractError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ed25519" => Ok(Self::Ed25519),
            "secp256k1" => Ok(Self::Secp256k1),
            _ => Err(ContractError::Other),
        }
    }
}
