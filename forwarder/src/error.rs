#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq, Eq))]
pub enum ContractError {
    ParseAccountError,
    BorshDeserializeError,
    BorshSerializeError,
    OneYoctoAttachError,
    PrivateCallError,
    BadUtf8String,
    BadNumber,
}

impl AsRef<[u8]> for ContractError {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::ParseAccountError => b"ERR_PARS_ACCOUNT",
            Self::BorshDeserializeError => b"ERR_BORCH_DESERIALIZE",
            Self::BorshSerializeError => b"ERR_BORCH_SERIALIZE",
            Self::OneYoctoAttachError => b"ERR_ONE_YOCTO_ATACH",
            Self::PrivateCallError => b"ERR_PRIVATE_CALL",
            Self::BadUtf8String => b"ERR_BAD_UTF8_STRING",
            Self::BadNumber => b"ERR_BAD_NUMBER",
        }
    }
}
