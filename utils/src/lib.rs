use near_sdk::{env, AccountId};

/// Creates a prefix for the forwarder account id.
#[must_use]
pub fn forwarder_prefix(
    address: &str,
    target_network: &AccountId,
    fees_contract_id: &AccountId,
) -> String {
    let address = address.trim_start_matches("0x");
    let bytes = [
        address.as_bytes(),
        target_network.as_bytes(),
        fees_contract_id.as_bytes(),
    ]
    .concat();
    near_sdk::bs58::encode(env::keccak256_array(&bytes))
        .into_string()
        .to_lowercase()
}
