use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use solana_sdk::account::Account;
use solana_sdk::clock::Epoch;
use solana_sdk::instruction::CompiledInstruction;
use solana_sdk::message::v0::MessageAddressTableLookup;
use solana_sdk::message::{legacy, v0, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use solana_transaction_status::{
    EncodableWithMeta, EncodedTransaction, UiMessage, UiRawMessage, UiTransaction,
};

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

fn from_json_tx(tx: UiTransaction) -> Result<VersionedTransaction, eyre::Error> {
    Ok(VersionedTransaction {
        signatures: tx
            .signatures
            .iter()
            .try_fold(Vec::default(), |mut sigs, sig| {
                sig.parse().map(|sig| {
                    sigs.push(sig);

                    sigs
                })
            })?,
        message: match tx.message {
            UiMessage::Parsed(_) => unimplemented!(),
            UiMessage::Raw(UiRawMessage {
                header,
                account_keys,
                recent_blockhash,
                instructions,
                address_table_lookups,
            }) => {
                let account_keys =
                    account_keys
                        .iter()
                        .try_fold(Vec::default(), |mut keys, key| {
                            key.parse().map(|key| {
                                keys.push(key);

                                keys
                            })
                        })?;
                let recent_blockhash = recent_blockhash.parse()?;
                let instructions =
                    instructions
                        .into_iter()
                        .try_fold(Vec::default(), |mut ixs, ix| {
                            solana_sdk::bs58::decode(&ix.data).into_vec().map(|data| {
                                ixs.push(CompiledInstruction {
                                    program_id_index: ix.program_id_index,
                                    accounts: ix.accounts,
                                    data,
                                });

                                ixs
                            })
                        })?;

                match address_table_lookups {
                    Some(address_table_lookups) => VersionedMessage::V0(v0::Message {
                        header,
                        account_keys,
                        recent_blockhash,
                        instructions,
                        address_table_lookups: address_table_lookups.into_iter().try_fold(
                            Vec::default(),
                            |mut alts, alt| {
                                alt.account_key.parse().map(|account_key| {
                                    alts.push(MessageAddressTableLookup {
                                        account_key,
                                        writable_indexes: alt.writable_indexes,
                                        readonly_indexes: alt.readonly_indexes,
                                    });

                                    alts
                                })
                            },
                        )?,
                    }),
                    None => VersionedMessage::Legacy(legacy::Message {
                        header,
                        account_keys,
                        recent_blockhash,
                        instructions,
                    }),
                }
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use solana_sdk::address_lookup_table::AddressLookupTableAccount;
    use solana_sdk::hash::Hash;
    use solana_sdk::message::VersionedMessage;
    use solana_sdk::signer::Signer;
    use solana_sdk::system_instruction;
    use solana_sdk::transaction::Transaction;

    use super::*;
    use crate::utils::test_payer_keypair;

    const DUMMY_PUBKEY: Pubkey = Pubkey::new_from_array([1; 32]);
    const DUMMY_HASH: Hash = Hash::new_from_array([2; 32]);

    #[serde_as]
    #[derive(Debug, Serialize, Deserialize)]
    struct Wrapper(#[serde_as(as = "TxAsJsonTx")] VersionedTransaction);

    #[test]
    fn round_trip_legacy_tx() {
        let tx = Wrapper(
            Transaction::new_signed_with_payer(
                &[system_instruction::transfer(&test_payer_keypair().pubkey(), &DUMMY_PUBKEY, 500)],
                Some(&test_payer_keypair().pubkey()),
                &[test_payer_keypair()],
                DUMMY_HASH,
            )
            .into(),
        );

        // Act - Serialize.
        let serialized = serde_json::to_string_pretty(&tx).unwrap();

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

        // Act - Recover.
        let recovered: Wrapper = serde_json::from_str(&serialized).unwrap();

        // Assert.
        assert_eq!(recovered.0, tx.0);
    }

    #[test]
    fn round_trip_v0_tx() {
        let message = solana_sdk::message::v0::Message::try_compile(
            &test_payer_keypair().pubkey(),
            &[system_instruction::transfer(&test_payer_keypair().pubkey(), &DUMMY_PUBKEY, 500)],
            &vec![AddressLookupTableAccount { key: DUMMY_PUBKEY, addresses: vec![DUMMY_PUBKEY] }],
            DUMMY_HASH,
        )
        .unwrap();
        let tx = Wrapper(
            VersionedTransaction::try_new(VersionedMessage::V0(message), &[test_payer_keypair()])
                .unwrap(),
        );

        // Act - Serialize.
        let serialized = serde_json::to_string_pretty(&tx).unwrap();

        // Assert.
        expect![[r#"
            {
              "signatures": [
                "44z9pb2J9mfT6VSj3hL2KmFUjU7BQub8xV1iyCTduaSVb83db1GL4gfbamAgrP2GB1yk7rkLHmHfsB1mgxouxDRH"
              ],
              "message": {
                "header": {
                  "numRequiredSignatures": 1,
                  "numReadonlySignedAccounts": 0,
                  "numReadonlyUnsignedAccounts": 1
                },
                "accountKeys": [
                  "AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9",
                  "11111111111111111111111111111111"
                ],
                "recentBlockhash": "8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR",
                "instructions": [
                  {
                    "programIdIndex": 1,
                    "accounts": [
                      0,
                      2
                    ],
                    "data": "3Bxs4hfoaMPsQgGf",
                    "stackHeight": null
                  }
                ],
                "addressTableLookups": [
                  {
                    "accountKey": "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi",
                    "writableIndexes": [
                      0
                    ],
                    "readonlyIndexes": []
                  }
                ]
              }
            }"#]]
        .assert_eq(&serialized);

        // Act - Recover.
        let recovered: Wrapper = serde_json::from_str(&serialized).unwrap();

        // Assert.
        assert_eq!(recovered.0, tx.0);
    }
}
