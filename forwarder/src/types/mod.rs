use borsh::BorshSerialize;

use crate::error::ContractError;

pub use account_id::AccountId;
pub use address::Address;
pub use promise::{
    PromiseAction, PromiseBatchAction, PromiseCreateArgs, PromiseResult, PromiseWithCallbackArgs,
};

pub type Vec<T> = arrayvec::ArrayVec<T, 256>;

mod account_id;
mod address;
mod promise;

pub fn to_borsh<T>(value: &T) -> Result<Vec<u8>, ContractError>
where
    T: BorshSerialize + ?Sized,
{
    let len = borsh::object_length(value).map_err(|_| ContractError::BorshSerializeError)?;
    let mut buf = Vec::new();
    assert!(len <= buf.capacity());
    unsafe {
        buf.set_len(len);
    }
    value
        .serialize(&mut buf.as_mut_slice())
        .map_err(|_| ContractError::BorshSerializeError)?;

    Ok(buf)
}
