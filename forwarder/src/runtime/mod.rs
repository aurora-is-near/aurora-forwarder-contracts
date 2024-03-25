use crate::runtime::sys::exports;
use crate::types::{AccountId, PromiseAction, PromiseBatchAction};

pub use env::Env;
pub use handler::PromiseHandler;
pub use io::{StorageIntermediate, IO};

mod env;
mod handler;
mod io;
mod sys;

/// Wrapper type for indices in NEAR's register API.
pub struct RegisterIndex(pub(crate) u64);

impl StorageIntermediate for RegisterIndex {
    fn len(&self) -> usize {
        unsafe {
            let result = exports::register_len(self.0);
            // By convention, an unused register will return a length of U64::MAX
            // (see https://nomicon.io/RuntimeSpec/Components/BindingsSpec/RegistersAPI).
            if result < u64::MAX {
                usize::try_from(result).unwrap_or_default()
            } else {
                0
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn copy_to_slice(&self, buffer: &mut [u8]) {
        unsafe { exports::read_register(self.0, buffer.as_ptr() as u64) }
    }
}

#[derive(Copy, Clone)]
pub struct Runtime;

impl Runtime {
    pub const READ_STORAGE_REGISTER_ID: RegisterIndex = RegisterIndex(0);
    pub const INPUT_REGISTER_ID: RegisterIndex = RegisterIndex(1);
    pub const WRITE_REGISTER_ID: RegisterIndex = RegisterIndex(2);
    pub const EVICT_REGISTER_ID: RegisterIndex = RegisterIndex(3);
    pub const ENV_REGISTER_ID: RegisterIndex = RegisterIndex(4);
    pub const PROMISE_REGISTER_ID: RegisterIndex = RegisterIndex(5);

    /// Assumes a valid account ID has been written to `ENV_REGISTER_ID`
    /// by a previous call.
    pub(crate) fn read_account_id() -> AccountId {
        let bytes = Self::ENV_REGISTER_ID.to_vec();
        let str = core::str::from_utf8(bytes.as_ref()).expect("Invalid UTF-8 string");
        AccountId::new(str).unwrap_or_default()
    }

    pub(crate) unsafe fn append_batch_actions<const S: usize>(
        id: u64,
        args: &PromiseBatchAction<S>,
    ) {
        for action in &args.actions {
            match action {
                PromiseAction::AddFullAccessKey { public_key, nonce } => {
                    let pk_bytes = public_key.as_slice();
                    exports::promise_batch_action_add_key_with_full_access(
                        id,
                        pk_bytes.len() as _,
                        pk_bytes.as_ptr() as _,
                        *nonce,
                    );
                }
            }
        }
    }
}

pub fn panic_utf8(bytes: &[u8]) -> ! {
    unsafe {
        exports::panic_utf8(bytes.len() as u64, bytes.as_ptr() as u64);
    }
    unreachable!()
}

pub trait SdkUnwrap<T> {
    fn sdk_unwrap(self) -> T;
}

impl<T, E: AsRef<[u8]>> SdkUnwrap<T> for Result<T, E> {
    fn sdk_unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => panic_utf8(e.as_ref()),
        }
    }
}

pub trait SdkExpect<T> {
    fn sdk_expect(self, msg: &str) -> T;
}

impl<T> SdkExpect<T> for Option<T> {
    fn sdk_expect(self, msg: &str) -> T {
        self.unwrap_or_else(|| panic_utf8(msg.as_bytes()))
    }
}
