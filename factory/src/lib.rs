use aurora_engine_types::types::Address;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Gas, NearToken, PanicOnDefault, Promise,
};

const FORWARDER_WASM: &[u8] = include_bytes!("../../res/aurora-forwarder.wasm");
const STORAGE_BALANCE_BOUND: NearToken = NearToken::from_yoctonear(1_250_000_000_000_000_000_000);
const FORWARDER_NEW_GAS: Gas = Gas::from_tgas(2);
const FORWARD_TOKENS_GAS: Gas = Gas::from_tgas(150);

pub const MAX_NUM_CONTRACTS: usize = 12;
pub const INIT_BALANCE: NearToken = NearToken::from_millinear(310);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct AuroraForwarderFactory {
    owner: AccountId,
    fees_contract_id: AccountId,
}

#[near_bindgen]
impl AuroraForwarderFactory {
    #[init]
    #[must_use]
    #[allow(clippy::use_self)]
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
    /// - list of parameters has more than `MAX_NUM_CONTRACTS` elements (consult implementation);
    /// - wrong parameters;
    #[must_use]
    #[private]
    pub fn create(&self, parameters: Vec<DeployParameters>) -> Vec<AccountId> {
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
                let args = borsh::to_vec(&ForwarderParameters {
                    target_address: Address::decode(params.target_address.trim_start_matches("0x"))
                        .unwrap(),
                    target_network: &params.target_network,
                    wnear_contract_id: &params.wnear_contract_id,
                    fees_contract_id: &self.fees_contract_id,
                    owner_id: &env::current_account_id(),
                })
                .expect("Couldn't create args");

                let _ = Promise::new(forwarder_id.clone())
                    .create_account()
                    .transfer(INIT_BALANCE)
                    .deploy_contract(FORWARDER_WASM.to_vec())
                    .function_call(
                        "new".to_string(),
                        args,
                        NearToken::from_near(0),
                        FORWARDER_NEW_GAS,
                    )
                    .then(
                        ext_token::ext(params.wnear_contract_id)
                            .with_attached_deposit(STORAGE_BALANCE_BOUND)
                            .storage_deposit(forwarder_id.clone()),
                    );

                forwarder_id
            })
            .collect::<Vec<_>>()
    }

    /// Forward tokens for a specific forwarder.
    #[private]
    pub fn forward_tokens(&mut self, forwarder_id: AccountId, token_id: AccountId) -> Promise {
        ext_forwarder::ext(forwarder_id)
            .with_static_gas(FORWARD_TOKENS_GAS)
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .forward(token_id)
    }

    /// Set new fees contract id.
    #[private]
    pub fn set_fees_contract_id(&mut self, fees_contract_id: AccountId) {
        self.fees_contract_id = fees_contract_id;
    }

    /// Return fees contract id.
    #[must_use]
    pub const fn get_fees_contract_id(&self) -> &AccountId {
        &self.fees_contract_id
    }

    /// Destroy forwarder.
    #[private]
    pub fn destroy_forwarder(&mut self, account_id: AccountId) -> Promise {
        ext_forwarder::ext(account_id).destroy()
    }
}

#[ext_contract(ext_token)]
pub trait ExtToken {
    fn storage_deposit(&self, account_id: AccountId);
}

#[ext_contract(ext_forwarder)]
pub trait ExtForwarder {
    fn forward(&self, #[serializer(borsh)] token_id: AccountId);
    fn destroy(&self);
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DeployParameters {
    pub target_address: String,
    pub target_network: AccountId,
    pub wnear_contract_id: AccountId,
}

#[derive(BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct ForwarderParameters<'a> {
    pub target_address: Address,
    pub target_network: &'a AccountId,
    pub wnear_contract_id: &'a AccountId,
    pub fees_contract_id: &'a AccountId,
    pub owner_id: &'a AccountId,
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
