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
    |tx: &VersionedTransaction| {
        match tx.json_encode() {
            EncodedTransaction::Json(tx) => tx,
            _ => unreachable!(),
        }
    },
    |tx: UiTransaction| -> Result<_, std::convert::Infallible> {
        Ok(EncodedTransaction::Json(tx).decode().unwrap())
    }
);

/*
/// A more efficient JSON representation of a [`VersionedTransaction`].
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonVersionedTransaction {
    #[serde_as(as = "Vec<serde_with::DisplayFromStr>")]
    pub signatures: Vec<Signature>,
    pub message: JsonVersionedMessage,
}

impl From<JsonVersionedTransaction> for VersionedTransaction {
    fn from(value: JsonVersionedTransaction) -> Self {
        VersionedTransaction {
            signatures: value.signatures,
            message: value.message,
        }
    }
}

impl From<VersionedTransaction> for JsonVersionedTransaction {
    fn from(value: VersionedTransaction) -> Self {
        JsonVersionedTransaction {
            signatures: value.signatures,
            message: value.message,
        }
    }
}


/// A more efficient JSON representation of a [`VersionedMessage`].
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum JsonVersionedMessage {
    Legacy(JsonLegacyMessage),
    V0(JsonMessageV0),
}

impl From<JsonVersionedMessage> for VersionedMessage {
    fn from(value: JsonVersionedMessage) -> Self {
        match value {
            JsonVersionedMessage::Legacy(message) => VersionedMessage::Legacy(message.into()),
            JsonVersionedMessage::V0(message) => VersionedMessage::V0(message.into()),
        }
    }
}

impl From<VersionedMessage> for JsonVersionedMessage {
    fn from(value: VersionedMessage) -> Self {
        match value {
            VersionedMessage::Legacy(message) => JsonVersionedMessage::Legacy(JsonLegacyMessage::from(message)),
            VersionedMessage::V0(message) => JsonVersionedMessage::V0(JsonLegacyV0::from(message)),
        }
    }
}


/// A more efficient JSON representation of a [`LegacyMessage`].
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct JsonLegacyMessage {
            message: (),
            is_writable_account_cache: (),
}

impl From<JsonLegacyMessage> for LegacyMessage<'static> {
    fn from(value: JsonLegacyMessage) -> Self {
        LegacyMessage {
            message: std::borrow::Cow::Owned(value.message.into()),
            is_writable_account_cache: value.is_writable_account_cache,
        }
    }
}

impl<'a> From<LegacyMessage<'a>> for JsonLegacyMessage {
    fn from(value: LegacyMessage<'a>) -> Self {
        JsonLegacyMessage {
            message: value.message,
            is_writable_account_cache: value.is_writable_account_cache,
        }
    }
}

/// A more efficient JSON representation of a [`legacy::Message`].
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct JsonMessage {
    a: UiTransactionEncoding
    header: MessageHeader,
    account_keys: Vec<Pubkey>,
    recent_blockhash: Hash,
    instructions: Vec<CompiledInstruction>,
}

impl From<JsonMessage> for Message {
    fn from(value: JsonMessage) -> Self {
        legacy::Message {
            header: value.header,
            account_keys: value.account_keys,
            recent_blockhash: value.recent_blockhash,
            instructions: value.instructions,
        }
    }
}

impl From<legacy::Message> for JsonMessage {
    fn from(value: legacy::Message) -> Self {
        JsonMessage {
            header: value.header,
            account_keys: value.account_keys,
            recent_blockhash: value.recent_blockhash,
            instructions: value.instructions,
        }
    }
}
*/
