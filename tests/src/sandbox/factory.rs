use aurora_forwarder_factory::DeployParameters;
use near_sdk::serde_json::json;
use near_workspaces::{AccountId, Contract};

pub trait Factory {
    async fn create(&self, params: Vec<DeployParameters>) -> anyhow::Result<Vec<AccountId>>;
}

impl Factory for Contract {
    async fn create(&self, params: Vec<DeployParameters>) -> anyhow::Result<Vec<AccountId>> {
        let result = self
            .call("create")
            .args_json(json!({
                "parameters": params
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        dbg!(&result);
        assert!(result.is_success());

        result.json().map_err(Into::into)
    }
}
