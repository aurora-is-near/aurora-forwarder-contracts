use crate::sandbox::{
    aurora::Aurora, forwarder::Forwarder, fungible_token::FungibleToken, Sandbox,
};
use near_workspaces::types::{AccessKeyPermission, NearToken, PublicKey};
use std::str::FromStr;

const RECEIVER: &str = "0x17ffdf6becbbc34d5c7d3bf4a0ed4a680395d057";
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
    let fees = sandbox.deploy_fee().await.unwrap();
    let result = sandbox
        .deploy_forwarder(
            aurora.id(),
            "0x17ffdf6becbbc34d5c7d3bf4a0ed4a680395d057",
            fees.id(),
        )
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
    let forward_amount = 1_000_000_000;
    let fee_percent = 5;
    let sandbox = Sandbox::new().await.unwrap();
    let (ft, ft_owner) = sandbox.deploy_ft(TOTAL_SUPPLY, "USDT", 6).await.unwrap();

    let aurora = sandbox.deploy_aurora().await.unwrap();
    ft.storage_deposit(aurora.id()).await.unwrap();

    let erc20 = aurora.deploy_erc20(ft.id()).await.unwrap();
    assert_eq!(erc20.balance_of(RECEIVER).await, 0);

    let fees = sandbox.deploy_fee().await.unwrap();
    ft.storage_deposit(fees.id()).await.unwrap();

    let forwarder = sandbox
        .deploy_forwarder(aurora.id(), RECEIVER, fees.id())
        .await
        .unwrap();
    ft.storage_deposit(forwarder.id()).await.unwrap();

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

    assert_eq!(erc20.balance_of(RECEIVER).await, balance);
    assert_eq!(ft.ft_balance_of(aurora.as_account()).await, balance);
    assert_eq!(ft.ft_balance_of(fees.as_account()).await, fee);
    assert_eq!(ft.ft_balance_of(forwarder.as_account()).await, 0);
    assert_eq!(
        ft.ft_balance_of(&ft_owner).await,
        TOTAL_SUPPLY - forward_amount
    );
}

#[allow(clippy::similar_names)]
#[tokio::test]
async fn test_forward_two_tokens() {
    let forward_amount = 1_000_000_000;
    let fee_percent = 5;
    let sandbox = Sandbox::new().await.unwrap();
    let (usdt, usdt_owner) = sandbox.deploy_ft(TOTAL_SUPPLY, "USDT", 6).await.unwrap();
    let (usdc, usdc_owner) = sandbox.deploy_ft(TOTAL_SUPPLY, "USDC", 6).await.unwrap();

    let aurora = sandbox.deploy_aurora().await.unwrap();
    usdt.storage_deposit(aurora.id()).await.unwrap();
    usdc.storage_deposit(aurora.id()).await.unwrap();

    let usdt_erc20 = aurora.deploy_erc20(usdt.id()).await.unwrap();
    let usdc_erc20 = aurora.deploy_erc20(usdc.id()).await.unwrap();
    assert_eq!(usdt_erc20.balance_of(RECEIVER).await, 0);
    assert_eq!(usdc_erc20.balance_of(RECEIVER).await, 0);

    let fees = sandbox.deploy_fee().await.unwrap();
    usdt.storage_deposit(fees.id()).await.unwrap();
    usdc.storage_deposit(fees.id()).await.unwrap();

    let forwarder = sandbox
        .deploy_forwarder(aurora.id(), RECEIVER, fees.id())
        .await
        .unwrap();
    usdt.storage_deposit(forwarder.id()).await.unwrap();
    usdc.storage_deposit(forwarder.id()).await.unwrap();

    usdt.ft_transfer(&usdt_owner, forwarder.as_account(), forward_amount)
        .await
        .unwrap();
    usdc.ft_transfer(&usdc_owner, forwarder.as_account(), forward_amount)
        .await
        .unwrap();

    assert_eq!(
        usdt.ft_balance_of(forwarder.as_account()).await,
        forward_amount
    );
    assert_eq!(
        usdc.ft_balance_of(forwarder.as_account()).await,
        forward_amount
    );
    assert_eq!(usdt.ft_balance_of(aurora.as_account()).await, 0);
    assert_eq!(usdc.ft_balance_of(aurora.as_account()).await, 0);

    forwarder.forward(usdt.id()).await.unwrap();
    forwarder.forward(usdc.id()).await.unwrap();

    let fee = (forward_amount * fee_percent) / 100;
    let balance = forward_amount - fee;

    // Check USDT
    assert_eq!(usdt_erc20.balance_of(RECEIVER).await, balance);
    assert_eq!(usdt.ft_balance_of(aurora.as_account()).await, balance);
    assert_eq!(usdt.ft_balance_of(fees.as_account()).await, fee);
    assert_eq!(usdt.ft_balance_of(forwarder.as_account()).await, 0);
    assert_eq!(
        usdt.ft_balance_of(&usdt_owner).await,
        TOTAL_SUPPLY - forward_amount
    );
    // Check USDC
    assert_eq!(usdc_erc20.balance_of(RECEIVER).await, balance);
    assert_eq!(usdc.ft_balance_of(aurora.as_account()).await, balance);
    assert_eq!(usdc.ft_balance_of(fees.as_account()).await, fee);
    assert_eq!(usdc.ft_balance_of(forwarder.as_account()).await, 0);
    assert_eq!(
        usdc.ft_balance_of(&usdc_owner).await,
        TOTAL_SUPPLY - forward_amount
    );
}

#[tokio::test]
async fn test_using_full_access_key() {
    let sandbox = Sandbox::new().await.unwrap();
    let pk = PublicKey::from_str("ed25519:BaiF3VUJf5pxB9ezVtzH4SejpdYc7EA3SqrKczsj1wno").unwrap();
    let silo_account_id = "some.silo.near".parse().unwrap();
    let fees_account_id = "fees.near".parse().unwrap();
    let forwarder = sandbox
        .deploy_forwarder(&silo_account_id, RECEIVER, &fees_account_id)
        .await
        .unwrap();
    let key = forwarder.view_access_key(&pk).await.unwrap();
    assert!(matches!(key.permission, AccessKeyPermission::FullAccess));
}
