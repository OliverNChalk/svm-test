use std::ops::Deref;

use futures::future::join_all;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

#[async_trait::async_trait]
pub trait FetchAccounts: Sync {
    async fn account(&self, key: &Pubkey) -> Account;
    async fn accounts(&self, keys: &[Pubkey]) -> Vec<Account> {
        join_all(keys.iter().map(|key| self.account(key))).await
    }
}

pub trait AccountLoader {
    /// Loads a single account.
    fn load(&self, key: &Pubkey) -> Account;

    /// Loads multiple accounts.
    ///
    /// ## Dev
    ///
    /// The default implementation simply calls [`Self::load`] in a loop.
    fn load_multiple(&self, keys: &[Pubkey]) -> Vec<Account> {
        keys.iter().map(|key| self.load(key)).collect()
    }
}

impl<L> AccountLoader for std::sync::Arc<L>
where
    L: AccountLoader,
{
    fn load(&self, key: &Pubkey) -> Account {
        self.deref().load(key)
    }

    fn load_multiple(&self, keys: &[Pubkey]) -> Vec<Account> {
        self.deref().load_multiple(keys)
    }
}

impl AccountLoader for std::collections::HashMap<Pubkey, Account> {
    fn load(&self, key: &Pubkey) -> Account {
        self.get(key).cloned().unwrap_or_default()
    }
}

impl AccountLoader for std::collections::BTreeMap<Pubkey, Account> {
    fn load(&self, key: &Pubkey) -> Account {
        self.get(key).cloned().unwrap_or_default()
    }
}
