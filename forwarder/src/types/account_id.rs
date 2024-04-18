use arrayvec::ArrayString;
use borsh::{io, BorshDeserialize, BorshSerialize};

use crate::error::ContractError;

const MIN_ACCOUNT_ID_LEN: usize = 2;
const MAX_ACCOUNT_ID_LEN: usize = 64;

#[derive(Default, Debug, Copy, Clone)]
pub struct AccountId(ArrayString<MAX_ACCOUNT_ID_LEN>);

impl AccountId {
    pub fn new(account_id: &str) -> Result<Self, ContractError> {
        Self::validate(account_id)?;
        Ok(Self(
            ArrayString::from(account_id).map_err(|_| ContractError::ParseAccountError)?,
        ))
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn validate(account_id: &str) -> Result<(), ContractError> {
        if account_id.len() < MIN_ACCOUNT_ID_LEN || account_id.len() > MAX_ACCOUNT_ID_LEN {
            Err(ContractError::ParseAccountError)
        } else {
            // Adapted from https://github.com/near/near-sdk-rs/blob/fd7d4f82d0dfd15f824a1cf110e552e940ea9073/near-sdk/src/environment/env.rs#L819

            // NOTE: We don't want to use Regex here, because it requires extra time to compile it.
            // The valid account ID regex is /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/
            // Instead the implementation is based on the previous character checks.

            // We can safely assume that last char was a separator.
            let mut last_char_is_separator = true;

            for c in account_id.bytes() {
                let current_char_is_separator = match c {
                    b'a'..=b'z' | b'0'..=b'9' => false,
                    b'-' | b'_' | b'.' => true,
                    _ => {
                        return Err(ContractError::ParseAccountError);
                    }
                };
                if current_char_is_separator && last_char_is_separator {
                    return Err(ContractError::ParseAccountError);
                }
                last_char_is_separator = current_char_is_separator;
            }

            (!last_char_is_separator)
                .then_some(())
                .ok_or(ContractError::ParseAccountError)
        }
    }
}

impl BorshDeserialize for AccountId {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let len = <u32 as borsh::BorshDeserialize>::deserialize_reader(reader)? as usize;
        if len < MIN_ACCOUNT_ID_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected a string at least 2 bytes long",
            ));
        }
        if len > MAX_ACCOUNT_ID_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected a string no more than 64 bytes long",
            ));
        }

        let mut buf = [0u8; MAX_ACCOUNT_ID_LEN];
        let buf = &mut buf[..len];
        reader.read_exact(buf)?;

        core::str::from_utf8(&buf[..len])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))
            .and_then(|s| {
                Self::new(s).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid data"))
            })
    }
}

impl BorshSerialize for AccountId {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        <str as borsh::BorshSerialize>::serialize(self.as_str(), writer)
    }
}

impl PartialEq for AccountId {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

#[test]
fn test_account_id() {
    let account_id = AccountId::new("test.near").unwrap();
    assert_eq!(account_id.as_str(), "test.near");

    let ser = crate::types::to_borsh(&account_id).unwrap();
    let expected = AccountId::try_from_slice(ser.as_slice()).unwrap();

    assert_eq!(account_id, expected);
}
