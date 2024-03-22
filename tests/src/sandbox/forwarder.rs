use near_workspaces::types::NearToken;
use near_workspaces::{AccountId, Contract};

pub trait Forwarder {
    async fn forward(&self, token_id: &AccountId) -> anyhow::Result<()>;
}

impl Forwarder for Contract {
    async fn forward(&self, token_id: &AccountId) -> anyhow::Result<()> {
        let result = self
            .call("forward")
            .args_borsh(token_id)
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await
            .unwrap();
        assert!(result.is_success());

        Ok(())
    }
}
