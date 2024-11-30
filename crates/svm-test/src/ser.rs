use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use solana_sdk::account::Account;
use solana_sdk::clock::Epoch;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::{EncodableWithMeta, EncodedTransaction, UiTransaction};

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

serde_with::serde_conv!(
    pub TxAsJsonTx,
    VersionedTransaction,
    to_json_tx,
    from_json_tx
);

fn to_json_tx(tx: &VersionedTransaction) -> UiTransaction {
    match tx.json_encode() {
        EncodedTransaction::Json(tx) => tx,
        _ => unreachable!(),
    }
}

fn from_json_tx(tx: UiTransaction) -> Result<VersionedTransaction, std::convert::Infallible> {
    Ok(EncodedTransaction::Json(tx).decode().unwrap())
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use solana_sdk::hash::Hash;
    use solana_sdk::signer::Signer;
    use solana_sdk::system_instruction;
    use solana_sdk::transaction::Transaction;

    use super::*;
    use crate::utils::test_payer_keypair;

    const DUMMY_PUBKEY: Pubkey = Pubkey::new_from_array([1; 32]);
    const DUMMY_HASH: Hash = Hash::new_from_array([2; 32]);

    #[test]
    fn serialize_legacy_transaction() {
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(&test_payer_keypair().pubkey(), &DUMMY_PUBKEY, 500)],
            Some(&test_payer_keypair().pubkey()),
            &[test_payer_keypair()],
            DUMMY_HASH,
        )
        .into();

        // Act - Serialize.
        let serialized = serde_json::to_string_pretty(&to_json_tx(&tx)).unwrap();

        // Assert.
        expect![[r#"
            {
              "signatures": [
                "Y5KX5txmP8TwgsD2yx43AeUeHnLwPDz3nYhz3xudPMDq5AKmowKk3r3qjsGSp1VFSFhcc5T1dN9x3mqjpRpV1Xi"
              ],
              "message": {
                "header": {
                  "numRequiredSignatures": 1,
                  "numReadonlySignedAccounts": 0,
                  "numReadonlyUnsignedAccounts": 1
                },
                "accountKeys": [
                  "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9",
                  "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi",
                  "11111111111111111111111111111111"
                ],
                "recentBlockhash": "8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR",
                "instructions": [
                  {
                    "programIdIndex": 2,
                    "accounts": [
                      0,
                      1
                    ],
                    "data": "3Bxs4hfoaMPsQgGf",
                    "stackHeight": null
                  }
                ]
              }
            }"#]]
        .assert_eq(&serialized);
    }
}
