use crate::sandbox::aurora::Aurora;
use crate::sandbox::forwarder::Forwarder;
use crate::sandbox::fungible_token::FungibleToken;
use crate::sandbox::Sandbox;
use near_workspaces::types::NearToken;

const BALANCE: NearToken = NearToken::from_near(10);

#[tokio::test]
async fn test_forward_wrap_near_tokens() {
    let transfer = NearToken::from_near(2);
    let sandbox = Sandbox::new().await.unwrap();
    let alice = sandbox.create_subaccount("alice", BALANCE).await.unwrap();
    let (wrap, wrap_owner) = sandbox.deploy_wrap_near().await.unwrap();
    let fees = sandbox.deploy_fees(&[&wrap.id()]).await.unwrap();
    let silo = sandbox.deploy_aurora("silo").await.unwrap();
    let erc20 = silo.deploy_erc20(wrap.id()).await.unwrap();
    let forwarder = sandbox
        .deploy_forwarder(silo.id(), super::RECEIVER, fees.id(), wrap.id())
        .await
        .unwrap();

    wrap.storage_deposit(alice.id()).await.unwrap();
    wrap.storage_deposit(fees.id()).await.unwrap();
    wrap.storage_deposit(silo.id()).await.unwrap();
    wrap.storage_deposit(forwarder.id()).await.unwrap();

    wrap.ft_transfer(&wrap_owner, alice.id(), transfer.as_yoctonear())
        .await
        .unwrap();
    assert_eq!(
        wrap.ft_balance_of(alice.id()).await,
        transfer.as_yoctonear()
    );

    wrap.ft_transfer(&alice, forwarder.id(), transfer.as_yoctonear())
        .await
        .unwrap();

    forwarder.forward(wrap.id()).await.unwrap();

    let fee = transfer.as_yoctonear() * 5 / 100;
    let deposit = transfer.as_yoctonear() - fee;

    assert_eq!(erc20.balance_of(super::RECEIVER).await, deposit);
    assert_eq!(wrap.ft_balance_of(silo.id()).await, deposit);
    assert_eq!(wrap.ft_balance_of(fees.id()).await, fee);
}
