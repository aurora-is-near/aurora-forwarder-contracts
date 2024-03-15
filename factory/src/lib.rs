use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{
    assert_self, env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise,
};

const FORWARDER_WASM: &[u8] = include_bytes!("../../res/aurora-forwarder.wasm");
const INIT_BALANCE: Balance = 1_530_000_000_000_000_000_000_000;
const STORAGE_BALANCE_BOUND: Balance = 1_250_000_000_000_000_000_000;
const FORWARDER_NEW_GAS: Gas = Gas(1_000_000_000_000);
const MAX_NUM_CONTRACTS: usize = 8;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AuroraForwarderFactory {
    owner: AccountId,
    fees_contract_id: AccountId,
}

#[near_bindgen]
impl AuroraForwarderFactory {
    #[init]
    #[must_use]
    pub fn new(fees_contract_id: AccountId) -> Self {
        Self {
            owner: env::predecessor_account_id(),
            fees_contract_id,
        }
    }

    /// Create a bunch of new forwarder contracts.
    ///
    /// # Panics
    ///
    /// The reasons for panic:
    /// - if call the transaction not owner;
    /// - list of parameters is empty;
    /// - list of parameters has more than 10 elements;
    /// - wrong parameters;
    #[must_use]
    pub fn create(&self, parameters: Vec<DeployParameters>) -> Vec<AccountId> {
        assert_self();
        assert!(!parameters.is_empty(), "Parameters can't be empty");
        assert!(
            parameters.len() <= MAX_NUM_CONTRACTS,
            "Number of contracts can't be greater than {MAX_NUM_CONTRACTS}"
        );

        parameters
            .into_iter()
            .map(|params| {
                let forwarder_id = create_forwarder_id(
                    &params.target_address,
                    &params.target_network,
                    &self.fees_contract_id,
                );
                let args = near_sdk::serde_json::to_vec(&json!({
                    "target_address": params.target_address,
                    "target_network": params.target_network,
                    "fees_contract_id": &self.fees_contract_id,
                    "wnear_contract_id": params.wnear_contract_id,
                }))
                .expect("Couldn't create args");
                let _ = Promise::new(forwarder_id.clone())
                    .create_account()
                    .transfer(INIT_BALANCE)
                    .deploy_contract(FORWARDER_WASM.to_vec())
                    .function_call("new".to_string(), args, 0, FORWARDER_NEW_GAS)
                    .then(
                        ext_token::ext(params.wnear_contract_id)
                            .with_attached_deposit(STORAGE_BALANCE_BOUND)
                            .storage_deposit(forwarder_id.clone()),
                    );

                forwarder_id
            })
            .collect::<Vec<_>>()
    }

    /// Set new fees contract id.
    pub fn set_fees_contract_id(&mut self, fees_contract_id: AccountId) {
        assert_self();
        self.fees_contract_id = fees_contract_id;
    }

    /// Return fees contract id.
    #[must_use]
    pub const fn get_fees_contract_id(&self) -> &AccountId {
        &self.fees_contract_id
    }
}

#[ext_contract(ext_token)]
pub trait ExtToken {
    fn storage_deposit(&self, account_id: AccountId);
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DeployParameters {
    pub target_address: String,
    pub target_network: AccountId,
    pub wnear_contract_id: AccountId,
}

fn create_forwarder_id(
    address: &str,
    network: &AccountId,
    fees_contract_id: &AccountId,
) -> AccountId {
    let prefix = forwarder_utils::forwarder_prefix(address, network, fees_contract_id);
    format!("{prefix}.{}", env::current_account_id())
        .parse()
        .unwrap()
}
