use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::state_exists;
use near_sdk::json_types::{U128, U64};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use std::str::FromStr;

// We multiply the percent to 100 here to get rid of the floating numbers.
const DEFAULT_PERCENT: U64 = U64(500); // 5%

#[near_bindgen]
#[derive(Debug, BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FeesCalculator {
    percent: U64,
    owner: AccountId,
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
    pub fn new() -> Self {
        assert!(!state_exists());

        Self {
            percent: DEFAULT_PERCENT,
            owner: env::predecessor_account_id(),
        }
    }

    /// Calculates and returns the fee for the corresponding token and Aurora Network.
    #[must_use]
    pub fn calculate_fees(
        &self,
        amount: U128,
        token_id: &AccountId,
        target_network: &AccountId,
        target_address: String,
    ) -> U128 {
        let _ = (token_id, target_network, target_address);

        u128::from(self.percent.0)
            .checked_mul(amount.0)
            .unwrap_or_default()
            .saturating_div(10000)
            .into()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn set_fee_percent(&mut self, percent: String) {
        match parse_percent(&percent) {
            Ok(value) => self.percent = value,
            Err(e) => env::panic_str(&format!("couldn't parse percent: {e}")),
        }
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn get_fee_percent(&self) -> String {
        format!("{:.2}", self.percent.0 as f64 / 100.0)
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_percent(percent: &str) -> Result<U64, ParseError> {
    let result = f64::from_str(percent).map_err(ParseError::ParseFloat)?;

    if result < 0.01 {
        Err(ParseError::TooLowPercent)
    } else if result > 10.0 {
        Err(ParseError::TooHighPercent)
    } else {
        Ok(U64((result * 100.0) as u64))
    }
}

#[derive(Debug)]
enum ParseError {
    ParseFloat(ParseFloatError),
    TooLowPercent,
    TooHighPercent,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[allow(deprecated)]
        let msg = match self {
            Self::ParseFloat(error) => error.description(),
            Self::TooLowPercent => "Provided percent is less than 0.01%",
            Self::TooHighPercent => "Provided percent is more than 10%",
        };

        f.write_str(msg)
    }
}

#[test]
fn test_parse_percent() {
    assert_eq!(parse_percent("10").unwrap(), 1000.into());
    assert_eq!(parse_percent("2").unwrap(), 200.into());
    assert_eq!(parse_percent("0.25").unwrap(), 25.into());
    assert_eq!(parse_percent("0.01").unwrap(), 1.into());
    assert!(matches!(
        parse_percent("0.009").err(),
        Some(ParseError::TooLowPercent)
    ));
    assert!(matches!(
        parse_percent("10.1").err(),
        Some(ParseError::TooHighPercent)
    ));
    assert!(matches!(
        parse_percent("hello").err(),
        Some(ParseError::ParseFloat(_))
    ));
}
