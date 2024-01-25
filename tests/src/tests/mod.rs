use crate::sandbox::{
    aurora::Aurora, forwarder::Forwarder, fungible_token::FungibleToken, Sandbox,
};
use near_workspaces::types::NearToken;

const TOTAL_SUPPLY: u128 = 1_000_000_000_000_000;

#[tokio::test]
async fn test_creating_ft() {
    let sandbox = Sandbox::new().await.unwrap();
    let (ft, ft_owner) = sandbox.deploy_ft(TOTAL_SUPPLY, "USDT", 6).await.unwrap();
    let owner_balance = ft.ft_balance_of(&ft_owner).await;
    assert_eq!(owner_balance, TOTAL_SUPPLY);
    let alice = sandbox.create_subaccount("alice", 10).await.unwrap();
    assert_eq!(
        sandbox.balance(alice.id()).await,
        NearToken::from_near(10).as_yoctonear()
    );
    ft.storage_deposit(alice.id()).await.unwrap();

    let alice_balance = ft.ft_balance_of(&alice).await;
    assert_eq!(alice_balance, 0);

    let transfer_result = ft.ft_transfer(&ft_owner, &alice, 50).await;
    assert!(transfer_result.is_ok());

    let alice_balance = ft.ft_balance_of(&alice).await;
    assert_eq!(alice_balance, 50);
}

#[tokio::test]
async fn test_creating_forwarder() {
    let sandbox = Sandbox::new().await.unwrap();
    let aurora = sandbox.deploy_aurora().await.unwrap();
    let result = sandbox
        .deploy_forwarder(aurora.id(), "0x17ffdf6becbbc34d5c7d3bf4a0ed4a680395d057")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_creating_erc20() {
    let sandbox = Sandbox::new().await.unwrap();
    let (ft, _) = sandbox.deploy_ft(TOTAL_SUPPLY, "USDC", 6).await.unwrap();
    let aurora = sandbox.deploy_aurora().await.unwrap();
    let erc20 = aurora.deploy_erc20(ft.id()).await.unwrap();
    assert_eq!(
        erc20.address(),
        "0x35c61bd8f7cb50410abded58646dbdd6c447d135"
    );
}

#[tokio::test]
async fn test_main_successful_flow() {
    let receiver = "0x17ffdf6becbbc34d5c7d3bf4a0ed4a680395d057";
    let forward_amount = 1_000_000_000;
    let fee_percent = 5;
    let sandbox = Sandbox::new().await.unwrap();
    let (ft, ft_owner) = sandbox.deploy_ft(TOTAL_SUPPLY, "USDT", 6).await.unwrap();

    let aurora = sandbox.deploy_aurora().await.unwrap();
    ft.storage_deposit(aurora.id()).await.unwrap();

    let erc20 = aurora.deploy_erc20(ft.id()).await.unwrap();
    assert_eq!(erc20.balance_of(receiver).await, 0);

    let forwarder = sandbox
        .deploy_forwarder(aurora.id(), receiver)
        .await
        .unwrap();
    ft.storage_deposit(forwarder.id()).await.unwrap();

    let fees = sandbox.deploy_fee().await.unwrap();
    ft.storage_deposit(fees.id()).await.unwrap();

    ft.ft_transfer(&ft_owner, forwarder.as_account(), forward_amount)
        .await
        .unwrap();

    assert_eq!(
        ft.ft_balance_of(forwarder.as_account()).await,
        forward_amount
    );
    assert_eq!(ft.ft_balance_of(aurora.as_account()).await, 0);

    forwarder.forward(ft.id()).await.unwrap();

    let fee = (forward_amount * fee_percent) / 100;
    let balance = forward_amount - fee;

    assert_eq!(erc20.balance_of(receiver).await, balance);
    assert_eq!(ft.ft_balance_of(aurora.as_account()).await, balance);
    assert_eq!(ft.ft_balance_of(fees.as_account()).await, fee);
    assert_eq!(ft.ft_balance_of(forwarder.as_account()).await, 0);
    assert_eq!(
        ft.ft_balance_of(&ft_owner).await,
        TOTAL_SUPPLY - forward_amount
    );
}
