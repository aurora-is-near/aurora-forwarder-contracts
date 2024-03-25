use crate::types::{AccountId, Vec};

pub struct PromiseCreateArgs {
    pub target_account_id: AccountId,
    pub method: &'static str,
    pub args: Vec<u8>,
    pub attached_balance: u128,
    pub attached_gas: u64,
}

pub struct PromiseWithCallbackArgs {
    pub base: PromiseCreateArgs,
    pub callback: PromiseCreateArgs,
}

pub struct PromiseBatchAction<const S: usize> {
    pub target_account_id: AccountId,
    pub actions: [PromiseAction; S],
}

pub enum PromiseAction {
    AddFullAccessKey { public_key: [u8; 33], nonce: u64 },
}

#[allow(clippy::large_enum_variant)]
pub enum PromiseResult {
    Successful(Vec<u8>),
    Failed,
    NotReady,
}
