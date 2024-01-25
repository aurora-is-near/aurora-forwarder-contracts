use crate::sandbox::erc20::Erc20;
use near_workspaces::{AccountId, Contract};

pub trait Aurora {
    async fn deploy_erc20(&self, fungible_token: &AccountId) -> anyhow::Result<Erc20>;
}

impl Aurora for Contract {
    async fn deploy_erc20(&self, fungible_token: &AccountId) -> anyhow::Result<Erc20> {
        let result = self
            .call("deploy_erc20_token")
            .args_borsh(fungible_token)
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());
        let bytes: Vec<u8> = result.borsh()?;
        let address = aurora_engine_types::types::Address::try_from_slice(&bytes).unwrap();

        Ok(Erc20::new(address, self.clone()))
    }
}
