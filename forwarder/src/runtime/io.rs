use crate::error::ContractError;
use crate::runtime::sys::exports;
use crate::runtime::{RegisterIndex, Runtime};
use crate::types::{to_borsh, Vec};
use borsh::{BorshDeserialize, BorshSerialize};

pub trait StorageIntermediate: Sized {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
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

    /// Return a value to an external process. In the case of wasm contracts on NEAR
    /// this corresponds to the return value from the contract method.
    fn return_output(&mut self, value: &[u8]);

    /// Read the value in storage at the given key, if any.
    fn read_storage(&self, key: &[u8]) -> Option<Self::StorageValue>;

    /// Check if there is a value in storage at the given key, but do not read the value.
    /// Equivalent to `self.read_storage(key).is_some()` but more efficient.
    fn storage_has_key(&self, key: &[u8]) -> bool;

    /// Write the given value to storage under the given key. Returns a reference to the old
    /// value stored at that key (if any).
    fn write_storage(&mut self, key: &[u8], value: &[u8]) -> Option<Self::StorageValue>;

    /// Write a `StorageIntermediate` to storage directly under the given key
    /// (without ever needing to load the value into memory).Returns a reference
    /// to the old value stored at that key (if any).
    fn write_storage_direct(
        &mut self,
        key: &[u8],
        value: Self::StorageValue,
    ) -> Option<Self::StorageValue>;

    /// Remove entry from storage and capture the value present at the given key (if any)
    fn remove_storage(&mut self, key: &[u8]) -> Option<Self::StorageValue>;

    /// Read the length of the bytes stored at the given key.
    fn read_storage_len(&self, key: &[u8]) -> Option<usize> {
        self.read_storage(key).map(|s| s.len())
    }

    /// Convenience function to read the input and deserialize the bytes using borsh.
    fn read_input_borsh<U: BorshDeserialize>(&self) -> Result<U, ContractError> {
        self.read_input().to_value()
    }

    /// Convenience function to store the input directly in storage under the
    /// given key (without ever loading it into memory).
    fn read_input_and_store(&mut self, key: &[u8]) {
        let value = self.read_input();
        self.write_storage_direct(key, value);
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

    fn return_output(&mut self, value: &[u8]) {
        unsafe {
            exports::value_return(value.len() as u64, value.as_ptr() as u64);
        }
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

    fn storage_has_key(&self, key: &[u8]) -> bool {
        unsafe { exports::storage_has_key(key.len() as _, key.as_ptr() as _) == 1 }
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

    fn write_storage_direct(
        &mut self,
        key: &[u8],
        value: Self::StorageValue,
    ) -> Option<Self::StorageValue> {
        unsafe {
            if exports::storage_write(
                key.len() as _,
                key.as_ptr() as _,
                u64::MAX,
                value.0,
                Self::WRITE_REGISTER_ID.0,
            ) == 1
            {
                Some(Self::WRITE_REGISTER_ID)
            } else {
                None
            }
        }
    }

    fn remove_storage(&mut self, key: &[u8]) -> Option<Self::StorageValue> {
        unsafe {
            if exports::storage_remove(key.len() as _, key.as_ptr() as _, Self::EVICT_REGISTER_ID.0)
                == 1
            {
                Some(Self::EVICT_REGISTER_ID)
            } else {
                None
            }
        }
    }
}
