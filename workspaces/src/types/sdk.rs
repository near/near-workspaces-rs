use std::convert::TryFrom;

use borsh::BorshDeserialize;

use crate::error::{Error, ErrorKind};

use super::PublicKey;

impl TryFrom<near_sdk::PublicKey> for PublicKey {
    type Error = Error;

    fn try_from(pk: near_sdk::PublicKey) -> Result<Self, Self::Error> {
        Self::try_from_slice(pk.as_bytes()).map_err(|e| ErrorKind::DataConversion.custom(e))
    }
}
