use crate::sandbox::aurora::Aurora;
use crate::sandbox::factory::Factory;
use crate::sandbox::fungible_token::FungibleToken;
use crate::sandbox::Sandbox;
use aurora_forwarder_factory::DeployParameters;
use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;

const BALANCE: NearToken = NearToken::from_near(10);

#[tokio::test]
async fn test_forward_native_tokens() {
    let transfer = NearToken::from_near(5);
    let sandbox = Sandbox::new().await.unwrap();
    let alice = sandbox.create_subaccount("alice", BALANCE).await.unwrap();
    let (wrap, wrap_owner) = sandbox.deploy_wrap_near().await.unwrap();
    let fees = sandbox.deploy_fees(&[wrap.id()]).await.unwrap();
    let silo = sandbox.deploy_aurora("silo").await.unwrap();
    let erc20 = silo.deploy_erc20(wrap.id()).await.unwrap();
    let factory = sandbox.deploy_factory(fees.id()).await.unwrap();

    let forwarder = factory
        .create(&[DeployParameters {
            target_address: super::RECEIVER.to_string(),
            target_network: silo.id().as_str().parse().unwrap(),
            wnear_contract_id: wrap.id().as_str().parse().unwrap(),
        }])
        .await
        .unwrap()
        .pop()
        .unwrap();

    wrap.storage_deposit(alice.id()).await.unwrap();
    wrap.storage_deposit(fees.id()).await.unwrap();
    wrap.storage_deposit(silo.id()).await.unwrap();
    wrap.storage_deposit(&forwarder).await.unwrap();

    let result = alice.transfer_near(&forwarder, transfer).await.unwrap();
    assert!(result.is_success());

    let rounder = 10u128.pow(21);
    let fwd_balance = sandbox.balance(&forwarder).await;
    assert_eq!(
        fwd_balance / rounder,
        transfer
            .checked_add(NearToken::from_millinear(1800))
            .unwrap()
            .as_yoctonear()
            / rounder
    );

    let result = wrap_owner
        .call(&forwarder, "forward")
        .args_json(json!({
            "token_id": "near"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    let fwd_balance = sandbox.balance(&forwarder).await;
    assert_eq!(fwd_balance / rounder, 1800);

    let fee = transfer.as_yoctonear() * 5 / 100;
    let deposit = transfer.as_yoctonear() - fee;

    assert_eq!(wrap.ft_balance_of(fees.id()).await / rounder, fee / rounder);
    assert_eq!(
        wrap.ft_balance_of(silo.id()).await / rounder,
        deposit / rounder
    );
    assert_eq!(
        erc20.balance_of(super::RECEIVER).await / rounder,
        deposit / rounder
    );
}
