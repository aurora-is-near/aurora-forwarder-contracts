use aurora_engine_types::parameters::engine::{
    CallArgs, FunctionCallArgsV2, SubmitResult, TransactionStatus,
};
use aurora_engine_types::types::Address;
use aurora_engine_types::U256;
use ethabi::Token;
use near_sdk::env::keccak256;
use near_workspaces::Contract;

pub struct Erc20 {
    address: Address,
    aurora: Contract,
}

impl Erc20 {
    pub const fn new(address: Address, aurora: Contract) -> Self {
        Self { address, aurora }
    }

    pub fn address(&self) -> String {
        format!("0x{}", self.address.encode())
    }

    pub async fn balance_of(&self, address: &str) -> u128 {
        let address = Address::decode(address.trim_start_matches("0x")).unwrap();
        let input = build_input("balanceOf(address)", &[Token::Address(address.raw())]);
        let near_result = self
            .aurora
            .call("call")
            .args_borsh(CallArgs::V2(FunctionCallArgsV2 {
                contract: self.address,
                value: [0; 32],
                input,
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(near_result.is_success());
        let evm_result: SubmitResult = near_result.borsh().unwrap();

        match evm_result.status {
            TransactionStatus::Succeed(bytes) => U256::from_big_endian(&bytes).as_u128(),
            other => panic!("Wrong EVM transaction status: {other:?}"),
        }
    }
}

fn build_input(str_selector: &str, inputs: &[Token]) -> Vec<u8> {
    let sel = get_selector(str_selector);
    let inputs = ethabi::encode(inputs);
    [sel.as_slice(), inputs.as_slice()].concat()
}

fn get_selector(str_selector: &str) -> Vec<u8> {
    keccak256(str_selector.as_bytes())[..4].to_vec()
}
