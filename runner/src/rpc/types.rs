use near_primitives::hash::CryptoHash;
use near_primitives::types::AccountId;

const ONE_NEAR: u128 = 10u128.pow(24);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NearBalance {
    yoctonear_amount: u128,
}

impl NearBalance {
    pub fn from_yoctonear(yoctonear_amount: u128) -> Self {
        Self { yoctonear_amount }
    }

    pub fn to_yoctonear(&self) -> u128 {
        self.yoctonear_amount
    }
}

impl std::fmt::Display for NearBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.yoctonear_amount == 0 {
            write!(f, "0 NEAR")
        } else if self.yoctonear_amount < ONE_NEAR / 1_000 {
            write!(
                f,
                "less than 0.001 NEAR ({} yoctoNEAR)",
                self.yoctonear_amount
            )
        } else {
            write!(
                f,
                "{}.{:0>3} NEAR",
                self.yoctonear_amount / ONE_NEAR,
                self.yoctonear_amount / (ONE_NEAR / 1_000) % 1_000
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountInfo {
    pub account_id: AccountId,
    pub block_height: u64,
    pub block_hash: CryptoHash,
    pub balance: NearBalance,
    pub stake: NearBalance,
    pub used_storage_bytes: u64,
}
