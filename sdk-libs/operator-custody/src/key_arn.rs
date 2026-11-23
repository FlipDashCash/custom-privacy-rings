use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::CustodyError;

/// A key ARN, never an alias, the reply key of each KMS call is compared with it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KeyArn(String);

impl KeyArn {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for KeyArn {
    type Error = CustodyError;

    fn try_from(arn: String) -> Result<Self, CustodyError> {
        let mut parts = arn.splitn(6, ':');
        match std::array::from_fn(|_| parts.next().unwrap_or_default()) {
            ["arn", "aws" | "aws-cn" | "aws-us-gov", "kms", region, account, resource]
                if is_region(region)
                    && account.len() == 12
                    && account.bytes().all(|byte| byte.is_ascii_digit())
                    && resource.strip_prefix("key/").is_some_and(is_key_id) =>
            {
                Ok(Self(arn))
            }
            _ => Err(CustodyError::KeyArn),
        }
    }
}

impl FromStr for KeyArn {
    type Err = CustodyError;

    fn from_str(arn: &str) -> Result<Self, CustodyError> {
        arn.to_owned().try_into()
    }
}

impl From<KeyArn> for String {
    fn from(arn: KeyArn) -> Self {
        arn.0
    }
}

impl fmt::Display for KeyArn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub(crate) fn is_region(region: &str) -> bool {
    !region.is_empty()
        && region
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn is_key_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
