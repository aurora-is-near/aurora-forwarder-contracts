use near_workspaces::AccountId;

pub fn forwarder_prefix(address: &str, target_network: &AccountId) -> String {
    let bytes = [address.as_bytes(), target_network.as_bytes()].concat();
    near_sdk::bs58::encode(near_sdk::env::keccak256_array(&bytes))
        .into_string()
        .to_lowercase()
}

#[test]
fn test_forwarder_prefix() {
    assert_eq!(
        &forwarder_prefix(
            "872a7faa3fd5c5129d0280b55d0639b840cb9f63",
            &"silo-1.near".parse().unwrap()
        ),
        "8kw8swcmunzuanqbluqfwym4q8dxqpyjkjs7qqwpbluq"
    );
    assert_eq!(
        &forwarder_prefix(
            "61fa6bbf21287633db939dc38f5d0e68f1083062",
            &"silo-2.near".parse().unwrap()
        ),
        "f4dlqigd5psykkz6kennmmvmdfq7fdetiuchemwmapnd"
    );
}
