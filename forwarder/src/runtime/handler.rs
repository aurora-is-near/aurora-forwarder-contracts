use crate::runtime::io::StorageIntermediate;
use crate::runtime::sys::exports;
use crate::runtime::Runtime;
use crate::types::{PromiseBatchAction, PromiseCreateArgs, PromiseResult, PromiseWithCallbackArgs};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct PromiseId(u64);

impl PromiseId {
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

pub trait PromiseHandler {
    fn promise_result(&self, index: u64) -> Option<PromiseResult>;

    /// # Safety
    /// Creating calls to other contracts using the Engine account is dangerous because
    /// it has special admin privileges (especially with itself), for example minting
    /// bridged tokens. Therefore, this function must be used with extreme caution to prevent
    /// security vulnerabilities. In particular, it must not be possible for users to execute
    /// arbitrary calls using the Engine.
    unsafe fn promise_create_call(&mut self, args: &PromiseCreateArgs) -> PromiseId;

    /// Combine more than one promise into one.
    /// # Safety
    /// Safe because of use `promise_create_call` function under the hood.
    unsafe fn promise_create_and_combine(&mut self, args: &[PromiseCreateArgs]) -> PromiseId;

    /// # Safety
    /// See note on `promise_create_call`.
    unsafe fn promise_attach_callback(
        &mut self,
        base: PromiseId,
        callback: &PromiseCreateArgs,
    ) -> PromiseId;

    /// # Safety
    /// See note on `promise_create_call`. Promise batches in particular must be used very
    /// carefully because they can take destructive actions such as deploying new contract
    /// code or adding/removing access keys.
    unsafe fn promise_create_batch<const S: usize>(
        &mut self,
        args: &PromiseBatchAction<S>,
    ) -> PromiseId;

    fn promise_return(&mut self, promise: PromiseId);

    /// # Safety
    /// See note on `promise_create_call`.
    unsafe fn promise_create_with_callback(&mut self, args: &PromiseWithCallbackArgs) -> PromiseId {
        let base = self.promise_create_call(&args.base);
        self.promise_attach_callback(base, &args.callback)
    }
}

impl PromiseHandler for Runtime {
    fn promise_result(&self, index: u64) -> Option<PromiseResult> {
        unsafe {
            match exports::promise_result(index, Self::PROMISE_REGISTER_ID.0) {
                0 => Some(PromiseResult::NotReady),
                1 => {
                    let bytes = Self::PROMISE_REGISTER_ID.to_vec();
                    Some(PromiseResult::Successful(bytes))
                }
                2 => Some(PromiseResult::Failed),
                _ => None,
            }
        }
    }

    unsafe fn promise_create_call(&mut self, args: &PromiseCreateArgs) -> PromiseId {
        let account_id = args.target_account_id.as_bytes();
        let method_name = args.method.as_bytes();
        let arguments = args.args.as_slice();
        let amount = args.attached_balance;
        let gas = args.attached_gas;

        let id = {
            exports::promise_create(
                account_id.len() as _,
                account_id.as_ptr() as _,
                method_name.len() as _,
                method_name.as_ptr() as _,
                arguments.len() as _,
                arguments.as_ptr() as _,
                core::ptr::addr_of!(amount) as _,
                gas,
            )
        };
        PromiseId::new(id)
    }

    unsafe fn promise_create_and_combine(&mut self, args: &[PromiseCreateArgs]) -> PromiseId {
        let ids = args
            .iter()
            .map(|args| self.promise_create_call(args))
            .collect::<arrayvec::ArrayVec<_, 2>>();
        let id = exports::promise_and(ids.as_ptr() as _, ids.len() as _);

        PromiseId::new(id)
    }

    unsafe fn promise_attach_callback(
        &mut self,
        base: PromiseId,
        callback: &PromiseCreateArgs,
    ) -> PromiseId {
        let account_id = callback.target_account_id.as_bytes();
        let method_name = callback.method.as_bytes();
        let arguments = callback.args.as_slice();
        let amount = callback.attached_balance;
        let gas = callback.attached_gas;

        let id = {
            exports::promise_then(
                base.raw(),
                account_id.len() as _,
                account_id.as_ptr() as _,
                method_name.len() as _,
                method_name.as_ptr() as _,
                arguments.len() as _,
                arguments.as_ptr() as _,
                core::ptr::addr_of!(amount) as _,
                gas,
            )
        };

        PromiseId::new(id)
    }

    unsafe fn promise_create_batch<const S: usize>(
        &mut self,
        args: &PromiseBatchAction<S>,
    ) -> PromiseId {
        let account_id = args.target_account_id.as_bytes();

        let id = { exports::promise_batch_create(account_id.len() as _, account_id.as_ptr() as _) };

        Self::append_batch_actions(id, args);

        PromiseId::new(id)
    }

    fn promise_return(&mut self, promise: PromiseId) {
        unsafe {
            exports::promise_return(promise.raw());
        }
    }
}
