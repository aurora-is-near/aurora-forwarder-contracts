use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, AccountId, IntoStorageKey, PanicOnDefault};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;

// We multiply percents to 100 here to get rid of the floating numbers.
const MIN_FEE_PERCENT: u64 = 1; // 0.01 %
const MAX_FEE_PERCENT: u64 = 1000; // 10 %
const DEFAULT_PERCENT: U64 = U64(500); // 5%

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FeesCalculator {
    percent: Option<U64>,
    owner: AccountId,
    supported_tokens: BTreeSet<AccountId>,
}

#[near_bindgen]
impl FeesCalculator {
    /// Contract's constructor.
    ///
    /// # Panics
    ///
    /// The constructor panics if the state already exists.
    #[init]
    #[must_use]
    pub fn new(tokens: Vec<AccountId>) -> Self {
        Self {
            percent: Some(DEFAULT_PERCENT),
            owner: env::predecessor_account_id(),
            supported_tokens: tokens.into_iter().collect(),
        }
    }

    /// Calculate and return the fee for the corresponding token and Aurora Network.
    ///
    /// # Panics
    ///
    /// There is a safe unwrap because we return 0 if the percent is None.
    #[must_use]
    pub fn calculate_fees(
        &self,
        amount: U128,
        token_id: &AccountId,
        target_network: &AccountId,
        target_address: String,
    ) -> U128 {
        let _ = (target_network, target_address);

        if self.percent.is_none() || !self.supported_tokens.contains(token_id) {
            0.into()
        } else {
            u128::from(self.percent.unwrap().0)
                .checked_mul(amount.0)
                .unwrap_or_default()
                .saturating_div(10000)
                .into()
        }
    }

    /// Set the percent of the fee.
    ///
    /// # Panics
    ///
    /// Panics if the invoker of the transaction is not owner.
    #[allow(clippy::needless_pass_by_value)]
    pub fn set_fee_percent(&mut self, percent: Option<String>) {
        assert_eq!(env::predecessor_account_id(), self.owner);

        match parse_percent(percent.as_deref()) {
            Ok(value) => self.percent = value,
            Err(e) => env::panic_str(&format!("Couldn't parse percent: {e}")),
        }
    }

    /// Returns current fee percent.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_fee_percent(&self) -> Option<String> {
        self.percent
            .map(|U64(v)| format!("{:.2}", v as f64 / 100.0))
    }

    /// Return a list of supported tokens.
    #[must_use]
    pub fn supported_tokens(&self) -> Vec<&AccountId> {
        self.supported_tokens.iter().collect()
    }

    /// Add an account id of a new supported NEP-141 token.
    ///
    /// # Panics
    ///
    /// Panic if the added token is already exist.
    pub fn add_supported_token(&mut self, token_id: AccountId) {
        assert!(
            self.supported_tokens.insert(token_id),
            "Token is already present"
        );
    }

    /// Remove the token from the list of supported.
    ///
    /// # Panics
    ///
    /// Panics if the removed token is not exists.
    pub fn remove_supported_token(&mut self, token_id: &AccountId) {
        assert!(
            self.supported_tokens.remove(token_id),
            "Nothing to remove, token: {token_id} hasn't been added"
        );
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
enum KeyPrefix {
    SupportedTokens,
}

impl IntoStorageKey for KeyPrefix {
    fn into_storage_key(self) -> Vec<u8> {
        match self {
            Self::SupportedTokens => b"supported_tokens".to_vec(),
        }
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_percent(percent: Option<&str>) -> Result<Option<U64>, ParseError> {
    let Some(percent) = percent else {
        return Ok(None);
    };

    validate_decimal_part(percent)?;

    let result = f64::from_str(percent)
        .map(|p| (p * 100.0) as u64) // as conversion is safe here because we validate the number of decimals
        .map_err(ParseError::ParseFloat)?;

    if result == 0 {
        Ok(None)
    } else if result < MIN_FEE_PERCENT {
        Err(ParseError::TooLowPercent)
    } else if result > MAX_FEE_PERCENT {
        Err(ParseError::TooHighPercent)
    } else {
        Ok(Some(U64(result)))
    }
}

#[derive(Debug)]
enum ParseError {
    ParseFloat(ParseFloatError),
    TooLowPercent,
    TooHighPercent,
    TooManyDecimals,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[allow(deprecated)]
        let msg = match self {
            Self::ParseFloat(error) => error.description(),
            Self::TooLowPercent => "provided percent is less than 0.01%",
            Self::TooHighPercent => "provided percent is more than 10%",
            Self::TooManyDecimals => "provided percent could contain only 2 decimals",
        };

        f.write_str(msg)
    }
}

fn validate_decimal_part(percent: &str) -> Result<(), ParseError> {
    match percent.split_once('.') {
        Some((_, decimal)) if decimal.len() > 2 => Err(ParseError::TooManyDecimals),
        _ => Ok(()), // no decimals or the number of decimals is less or equal 2.
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_percent, FeesCalculator, ParseError};
    use near_sdk::AccountId;

    #[test]
    fn test_parse_percent() {
        assert_eq!(parse_percent(None).unwrap(), None);
        assert_eq!(parse_percent(Some("0")).unwrap(), None);
        assert_eq!(parse_percent(Some("10")).unwrap(), Some(1000.into()));
        assert_eq!(parse_percent(Some("2")).unwrap(), Some(200.into()));
        assert_eq!(parse_percent(Some("0.25")).unwrap(), Some(25.into()));
        assert_eq!(parse_percent(Some("0.01")).unwrap(), Some(1.into()));
        assert!(matches!(
            parse_percent(Some("0.015")).err(),
            Some(ParseError::TooManyDecimals)
        ));
        assert!(matches!(
            parse_percent(Some("0.009")).err(),
            Some(ParseError::TooManyDecimals)
        ));
        assert!(matches!(
            parse_percent(Some("10.1")).err(),
            Some(ParseError::TooHighPercent)
        ));
        assert!(matches!(
            parse_percent(Some("hello")).err(),
            Some(ParseError::ParseFloat(_))
        ));
    }

    #[test]
    fn test_check_supported_tokens() {
        let aurora = "aurora".parse().unwrap();
        let target_address = "0xea2342".to_string();
        let usdt = "usdt.near".parse().unwrap();
        let mut contract = FeesCalculator::new(vec![]);

        assert_eq!(
            contract.calculate_fees(1000.into(), &usdt, &aurora, target_address.clone()),
            0.into() // we don't support the `usdt.near` yet, so we get 0 here
        );

        contract.add_supported_token(usdt.clone());

        assert_eq!(
            contract.calculate_fees(1000.into(), &usdt, &aurora, target_address.clone()),
            50.into()
        );

        contract.remove_supported_token(&usdt);

        assert_eq!(
            contract.calculate_fees(1000.into(), &usdt, &aurora, target_address),
            0.into()
        );
    }

    #[test]
    fn test_check_set_fee() {
        let aurora = "aurora".parse().unwrap();
        let target_address = "0xea2342".to_string();
        let usdt: AccountId = "usdt.near".parse().unwrap();
        let mut contract = FeesCalculator::new(vec![usdt.clone()]);

        assert_eq!(
            contract.calculate_fees(1000.into(), &usdt, &aurora, target_address.clone()),
            50.into()
        );

        contract.set_fee_percent(Some("0".to_string()));

        assert_eq!(
            contract.calculate_fees(1000.into(), &usdt, &aurora, target_address.clone()),
            0.into()
        );

        contract.set_fee_percent(Some("2.5".to_string()));

        assert_eq!(
            contract.calculate_fees(1000.into(), &usdt, &aurora, target_address),
            25.into()
        );
    }

    #[test]
    fn test_set_percent() {
        let mut contract = FeesCalculator::new(vec![]);

        assert_eq!(contract.get_fee_percent(), Some("5.00".to_string()));
        contract.set_fee_percent(Some("6".to_string()));
        assert_eq!(contract.get_fee_percent(), Some("6.00".to_string()));
        contract.set_fee_percent(Some("7.5".to_string()));
        assert_eq!(contract.get_fee_percent(), Some("7.50".to_string()));
        contract.set_fee_percent(Some("0".to_string()));
        assert_eq!(contract.get_fee_percent(), None);
    }

    #[test]
    #[should_panic(
        expected = "Couldn't parse percent: provided percent could contain only 2 decimals"
    )]
    fn test_set_percent_with_many_decimals() {
        let mut contract = FeesCalculator::new(vec![]);
        contract.set_fee_percent(Some("6.123".to_string()));
    }

    #[test]
    #[should_panic(expected = "Couldn't parse percent: provided percent is more than 10%")]
    fn test_set_too_high_percents() {
        let mut contract = FeesCalculator::new(vec![]);
        contract.set_fee_percent(Some("12.12".to_string()));
    }
}
