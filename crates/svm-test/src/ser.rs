use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use solana_sdk::account::Account;
use solana_sdk::clock::Epoch;
use solana_sdk::pubkey::Pubkey;

serde_with::serde_conv!(
    pub AccountAsJsonAccount,
    Account,
    |account: &Account| { JsonAccount::from(account.clone()) },
    |account: JsonAccount| -> Result<_, std::convert::Infallible> { Ok(account.into()) }
);

/// A more efficient JSON representation of a solana Account.
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonAccount {
    #[serde(default)]
    pub lamports: u64,
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Vec<u8>,
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub owner: Pubkey,
    #[serde(default)]
    pub executable: bool,
    #[serde(default)]
    pub rent_epoch: Epoch,
}

impl From<JsonAccount> for Account {
    fn from(value: JsonAccount) -> Self {
        Account {
            lamports: value.lamports,
            data: value.data,
            owner: value.owner,
            executable: value.executable,
            rent_epoch: value.rent_epoch,
        }
    }
}

impl From<Account> for JsonAccount {
    fn from(value: Account) -> Self {
        JsonAccount {
            lamports: value.lamports,
            data: value.data,
            owner: value.owner,
            executable: value.executable,
            rent_epoch: value.rent_epoch,
        }
    }
}
