use crate::error::ContractError;
use crate::runtime::sys::exports;
use crate::runtime::Runtime;
use crate::types::AccountId;

/// Returns information about the NEAR context in which the
/// transaction is executing. In the case of a standalone binary,
/// independent of NEAR these values would need to be mocked or otherwise
/// passed in from an external source.
pub trait Env {
    /// Account ID that signed the transaction.
    fn signer_account_id(&self) -> AccountId;
    /// Account ID of the currently executing contract.
    fn current_account_id(&self) -> AccountId;
    /// Account ID which called the current contract.
    fn predecessor_account_id(&self) -> AccountId;
    /// Height of the current block.
    fn block_height(&self) -> u64;
    /// Amount of NEAR attached to current call
    fn attached_deposit(&self) -> u128;
    /// Account's balance in yoctoNEAR.
    fn account_balance(&self) -> u128;

    fn assert_private_call(&self) -> Result<(), ContractError> {
        if self.predecessor_account_id() == self.current_account_id() {
            Ok(())
        } else {
            Err(ContractError::PrivateCallError)
        }
    }

    fn assert_one_yocto(&self) -> Result<(), ContractError> {
        if self.attached_deposit() == 1 {
            Ok(())
        } else {
            Err(ContractError::OneYoctoAttachError)
        }
    }
}

impl Env for Runtime {
    fn signer_account_id(&self) -> AccountId {
        unsafe {
            exports::signer_account_id(Self::ENV_REGISTER_ID.0);
        }
        Self::read_account_id()
    }

    fn current_account_id(&self) -> AccountId {
        unsafe {
            exports::current_account_id(Self::ENV_REGISTER_ID.0);
        }
        Self::read_account_id()
    }

    fn predecessor_account_id(&self) -> AccountId {
        unsafe {
            exports::predecessor_account_id(Self::ENV_REGISTER_ID.0);
        }
        Self::read_account_id()
    }

    fn block_height(&self) -> u64 {
        unsafe { exports::block_index() }
    }

    fn attached_deposit(&self) -> u128 {
        unsafe {
            let data = [0u8; core::mem::size_of::<u128>()];
            exports::attached_deposit(data.as_ptr() as u64);
            u128::from_le_bytes(data)
        }
    }

    fn account_balance(&self) -> u128 {
        unsafe {
            let data = [0u8; core::mem::size_of::<u128>()];
            exports::account_balance(data.as_ptr() as u64);
            u128::from_le_bytes(data)
        }
    }
}
