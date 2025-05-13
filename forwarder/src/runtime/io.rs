use crate::error::ContractError;
use crate::runtime::sys::exports;
use crate::runtime::{RegisterIndex, Runtime};
use crate::types::{to_borsh, Vec};
use borsh::{BorshDeserialize, BorshSerialize};

pub trait StorageIntermediate: Sized {
    fn len(&self) -> usize;
    fn copy_to_slice(&self, buffer: &mut [u8]);

    fn to_vec(&self) -> Vec<u8> {
        let len = self.len();
        let mut buf = Vec::new();
        assert!(len <= buf.capacity());
        unsafe {
            buf.set_len(len);
        }
        self.copy_to_slice(&mut buf[..len]);
        buf
    }

    fn to_value<T: BorshDeserialize>(&self) -> Result<T, ContractError> {
        let bytes = self.to_vec();
        T::try_from_slice(&bytes[..]).map_err(|_| ContractError::BorshDeserializeError)
    }
}

/// Trait for reading/writing values from storage and a generalized `stdin`/`stdout`.
pub trait IO {
    /// A type giving a reference to a value obtained by IO without loading it
    /// into memory. For example, in the case of a wasm contract on NEAR this
    /// will correspond to a register index.
    type StorageValue: StorageIntermediate;

    /// Read bytes that were passed as input to the process. This can be thought of as a
    /// generalization of `stdin` or command-line arguments. In the case of wasm contracts
    /// on NEAR these would be the arguments to the method.
    fn read_input(&self) -> Self::StorageValue;

    /// Read the value in storage at the given key, if any.
    fn read_storage(&self, key: &[u8]) -> Option<Self::StorageValue>;

    /// Write the given value to storage under the given key. Returns a reference to the old
    /// value stored at that key (if any).
    fn write_storage(&mut self, key: &[u8], value: &[u8]) -> Option<Self::StorageValue>;

    /// Convenience function to read the input and deserialize the bytes using borsh.
    fn read_input_borsh<U: BorshDeserialize>(&self) -> Result<U, ContractError> {
        self.read_input().to_value()
    }

    fn write_borsh<T: BorshSerialize>(
        &mut self,
        key: &[u8],
        value: &T,
    ) -> Option<Self::StorageValue> {
        let bytes = to_borsh(&value).ok()?;
        self.write_storage(key, &bytes[..])
    }
}

impl IO for Runtime {
    type StorageValue = RegisterIndex;

    fn read_input(&self) -> Self::StorageValue {
        unsafe {
            exports::input(Self::INPUT_REGISTER_ID.0);
        }
        Self::INPUT_REGISTER_ID
    }

    fn read_storage(&self, key: &[u8]) -> Option<Self::StorageValue> {
        unsafe {
            if exports::storage_read(
                key.len() as u64,
                key.as_ptr() as u64,
                Self::READ_STORAGE_REGISTER_ID.0,
            ) == 1
            {
                Some(Self::READ_STORAGE_REGISTER_ID)
            } else {
                None
            }
        }
    }

    fn write_storage(&mut self, key: &[u8], value: &[u8]) -> Option<Self::StorageValue> {
        unsafe {
            if exports::storage_write(
                key.len() as u64,
                key.as_ptr() as u64,
                value.len() as u64,
                value.as_ptr() as u64,
                Self::WRITE_REGISTER_ID.0,
            ) == 1
            {
                Some(Self::WRITE_REGISTER_ID)
            } else {
                None
            }
        }
    }
}
