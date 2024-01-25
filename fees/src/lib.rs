use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::state_exists;
use near_sdk::json_types::U128;
use near_sdk::store::LookupMap;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::str::FromStr;

const PREFIX: &[u8] = b"FEES";

#[near_bindgen]
#[derive(Debug, BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FeesCalculator {
    fees: LookupMap<AccountId, U128>,
    owner: AccountId,
}

#[near_bindgen]
impl FeesCalculator {
    /// Contract's constructor.
    ///
    /// # Panics
    ///
    /// The constructor panics if the state already exists.
    #[init]
    #[must_use]
    pub fn new() -> Self {
        assert!(!state_exists());
        let mut fees = LookupMap::new(PREFIX);
        fees.insert(
            AccountId::from_str("usdt.test.near").unwrap(),
            U128::from(10),
        );
        fees.insert(
            AccountId::from_str("usdc.test.near").unwrap(),
            U128::from(5),
        );
        fees.insert(
            AccountId::from_str("usdte.test.near").unwrap(),
            U128::from(10),
        );
        fees.insert(
            AccountId::from_str("usdce.test.near").unwrap(),
            U128::from(10),
        );

        Self {
            fees,
            owner: env::predecessor_account_id(),
        }
    }

    /// Calculates and returns the fee for the corresponding token and Aurora Network.
    pub fn calculate_fees(
        &self,
        amount: U128,
        token_id: &AccountId,
        target_network: &AccountId,
    ) -> U128 {
        let _ = target_network;
        let percent = self
            .fees
            .get(token_id)
            .copied()
            .unwrap_or_else(|| U128::from(0));

        percent
            .0
            .checked_mul(amount.0)
            .unwrap_or_default()
            .saturating_div(100)
            .into()
    }
}
