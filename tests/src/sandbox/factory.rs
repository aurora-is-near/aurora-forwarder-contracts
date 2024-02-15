use aurora_forwarder_factory::DeployParameters;
use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::{AccountId, Contract};

pub trait Factory {
    async fn create(&self, params: &[DeployParameters]) -> anyhow::Result<Vec<AccountId>>;
    async fn forward(&self, forwarder_id: &AccountId, token_id: &AccountId) -> anyhow::Result<()>;
}

impl Factory for Contract {
    async fn create(&self, params: &[DeployParameters]) -> anyhow::Result<Vec<AccountId>> {
        let result = self
            .call("create")
            .args_json(json!({
                "parameters": params
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success());

        result.json().map_err(Into::into)
    }

    async fn forward(&self, forwarder_id: &AccountId, token_id: &AccountId) -> anyhow::Result<()> {
        let result = self
            .as_account()
            .call(forwarder_id, "forward")
            .args_json(json!({
                "token_id": token_id
            }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success());

        Ok(())
    }
}
