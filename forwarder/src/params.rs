use arrayvec::ArrayString;
use borsh::{BorshDeserialize, BorshSerialize};
use core::str::FromStr;

use crate::error::ContractError;
use crate::runtime::{StorageIntermediate, IO};
use crate::types::{AccountId, Address, Vec};

const STATE_STORAGE_KEY: &[u8] = b"FWD_STATE";

#[derive(BorshSerialize, BorshDeserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(PartialEq, Debug))]
pub struct State {
    pub target_address: Address,
    pub target_network: AccountId,
    pub wnear_contract_id: AccountId,
    pub fees_contract_id: AccountId,
}

impl State {
    pub fn save<I: IO>(&self, io: &mut I) {
        io.write_borsh(STATE_STORAGE_KEY, self);
    }

    pub fn load<I: IO>(io: &I) -> Option<Self> {
        let data = io.read_storage(STATE_STORAGE_KEY)?.to_vec();
        Self::try_from_slice(data.as_slice()).ok()
    }
}

#[derive(BorshSerialize)]
pub struct FeesParams<'a> {
    pub amount: u128,
    pub token_id: &'a AccountId,
    pub target_network: &'a AccountId,
    pub target_address: Address,
}

#[derive(BorshDeserialize)]
pub struct ForwardParams {
    pub token_id: AccountId,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct FinishForwardParams {
    pub amount: u128,
    pub token_id: AccountId,
    pub promise_idx: u64,
}

pub fn ft_transfer_call_args(receiver_id: &AccountId, amount: u128, address: Address) -> Vec<u8> {
    let mut result = ArrayString::<128>::new();

    result.push_str(r#"{"receiver_id":""#);
    result.push_str(receiver_id.as_str());
    result.push_str(r#"","amount":""#);
    result.push_str(amount_to_str(amount).as_str());
    result.push_str(r#"","msg":""#);

    let mut address_str = ArrayString::<40>::zero_filled();

    unsafe {
        hex::encode_to_slice(address.0, address_str.as_bytes_mut()).unwrap();
    }

    result.push_str(address_str.as_str());
    result.push_str(r#""}"#);

    Vec::try_from(result.as_bytes()).unwrap_or_default()
}

pub fn ft_transfer_args(receiver_id: &AccountId, amount: u128) -> Vec<u8> {
    let mut result = ArrayString::<128>::new();

    result.push_str(r#"{"receiver_id":""#);
    result.push_str(receiver_id.as_str());
    result.push_str(r#"","amount":""#);
    result.push_str(amount_to_str(amount).as_str());
    result.push_str(r#""}"#);

    Vec::try_from(result.as_bytes()).unwrap_or_default()
}

pub fn ft_balance_args(account_id: &AccountId) -> Vec<u8> {
    let mut result = ArrayString::<128>::new();

    result.push_str(r#"{"account_id":""#);
    result.push_str(account_id.as_str());
    result.push_str(r#""}"#);

    Vec::try_from(result.as_bytes()).unwrap_or_default()
}

fn amount_to_str(mut amount: u128) -> ArrayString<39> {
    let mut len = 0;
    let mut buf = ['0'; 39];

    if amount == 0 {
        buf[0] = core::char::from_digit(0, 10).unwrap();
        len = 1;
    } else {
        while amount > 0 {
            let digit = amount % 10;
            buf[len] = core::char::from_digit(digit as u32, 10).unwrap();
            amount /= 10;
            len += 1;
        }
        buf[..len].reverse();
    }

    let mut result = ArrayString::<39>::new();
    buf[..len].iter().for_each(|d| result.push(*d));

    result
}

pub fn vec_to_number<T: FromStr>(vec: &Vec<u8>) -> Result<T, ContractError> {
    let x = core::str::from_utf8(&vec[..]).map_err(|_| ContractError::BadUtf8String)?;
    T::from_str(x.trim_matches('"')).map_err(|_| ContractError::BadNumber)
}

#[test]
fn test_deserialize_state() {
    let original = State {
        target_address: Address([1; 20]),
        target_network: AccountId::new("target.near").unwrap(),
        wnear_contract_id: AccountId::new("wnear.near").unwrap(),
        fees_contract_id: AccountId::new("fees.near").unwrap(),
    };

    let bytes = crate::types::to_borsh(&original).unwrap();
    let expected = State::try_from_slice(bytes.as_slice()).unwrap();

    assert_eq!(original, expected);
}

#[test]
fn test_ft_balance_args() {
    let json = ft_balance_args(&AccountId::new("test.near").unwrap());
    assert_eq!(&json[..], br#"{"account_id":"test.near"}"#);
}

#[test]
fn test_ft_transfer_args() {
    let json = ft_transfer_args(&AccountId::new("test.near").unwrap(), 12_345_670);
    assert_eq!(
        &json[..],
        br#"{"receiver_id":"test.near","amount":"12345670"}"#
    );

    let json = ft_transfer_args(&AccountId::new("test.near").unwrap(), 0);
    assert_eq!(&json[..], br#"{"receiver_id":"test.near","amount":"0"}"#);
}

#[test]
fn test_ft_transfer_call_args() {
    let mut address = [0; 20];
    hex::decode_to_slice("7e5f4552091a69125d5dfcb7b8c2659029395bdf", &mut address).unwrap();
    let json = ft_transfer_call_args(
        &AccountId::new("test.near").unwrap(),
        12_345_670,
        Address(address),
    );
    assert_eq!(&json[..], br#"{"receiver_id":"test.near","amount":"12345670","msg":"7e5f4552091a69125d5dfcb7b8c2659029395bdf"}"#);
}

#[test]
fn test_amount_to_str() {
    assert_eq!(amount_to_str(0).as_str(), "0");
    assert_eq!(amount_to_str(3_498_832).as_str(), "3498832");
    assert_eq!(
        amount_to_str(u128::MAX).as_str(),
        "340282366920938463463374607431768211455"
    );
}

#[test]
fn test_vec_to_number() {
    assert_eq!(
        vec_to_number(&Vec::try_from(b"42".as_slice()).unwrap()),
        Ok(42)
    );
    assert_eq!(
        vec_to_number::<u32>(&Vec::try_from(b"4\xc3\x28".as_slice()).unwrap()),
        Err(ContractError::BadUtf8String)
    );
    assert_eq!(
        vec_to_number::<u32>(&Vec::try_from(b"4x".as_slice()).unwrap()),
        Err(ContractError::BadNumber)
    );
}
