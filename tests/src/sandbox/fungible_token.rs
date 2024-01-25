use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use std::str::FromStr;

const STORAGE_DEPOSIT: NearToken = NearToken::from_yoctonear(2_350_000_000_000_000_000_000);

pub trait FungibleToken {
    async fn ft_balance_of(&self, account: &Account) -> u128;
    async fn ft_transfer(&self, from: &Account, to: &Account, amount: u128) -> anyhow::Result<()>;
    async fn storage_deposit(&self, account_id: &AccountId) -> anyhow::Result<()>;
}

impl FungibleToken for Contract {
    async fn ft_balance_of(&self, account: &Account) -> u128 {
        let result = self
            .view("ft_balance_of")
            .args_json(json!({
                "account_id": account.id()
            }))
            .await
            .unwrap();
        let value: String = result.json().unwrap();
        u128::from_str(&value).unwrap()
    }

    async fn ft_transfer(&self, from: &Account, to: &Account, amount: u128) -> anyhow::Result<()> {
        let result = from
            .call(self.id(), "ft_transfer")
            .args_json(json!({ "receiver_id": to.id(), "amount": amount.to_string() }))
            .deposit(NearToken::from_yoctonear(1))
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());
        Ok(())
    }

    async fn storage_deposit(&self, account_id: &AccountId) -> anyhow::Result<()> {
        let result = self
            .call("storage_deposit")
            .args_json(json!({"account_id": account_id }))
            .deposit(STORAGE_DEPOSIT)
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());
        Ok(())
    }
}
