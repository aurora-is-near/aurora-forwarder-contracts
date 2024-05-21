use aurora_forwarder_factory::DeployParameters;
use near_sdk::serde_json::json;
use near_workspaces::{AccountId, Contract};

pub trait Factory {
    async fn create(&self, params: &[DeployParameters]) -> anyhow::Result<Vec<AccountId>>;
    async fn forward(&self, forwarder_id: &AccountId, token_id: &AccountId) -> anyhow::Result<()>;
    async fn destroy(&self, forwarder_id: &AccountId) -> anyhow::Result<()>;
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
            .call("forward_tokens")
            .args_json(json!({
                "forwarder_id": forwarder_id,
                "token_id": token_id
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success());

        Ok(())
    }

    async fn destroy(&self, account_id: &AccountId) -> anyhow::Result<()> {
        let result = self
            .call("destroy_forwarder")
            .args_json(json!({
                "account_id": account_id
            }))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success());

        Ok(())
    }
}
