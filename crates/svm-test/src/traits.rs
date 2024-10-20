use std::ops::Deref;

use auto_impl::auto_impl;
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

#[async_trait::async_trait]
impl<T> FetchAccounts for std::sync::Arc<T>
where
    T: FetchAccounts + Send,
{
    async fn account(&self, key: &Pubkey) -> Account {
        self.deref().account(key).await
    }

    async fn accounts(&self, keys: &[Pubkey]) -> Vec<Account> {
        self.deref().accounts(keys).await
    }
}

#[auto_impl(Arc, Box)]
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

impl<S> AccountLoader for std::collections::HashMap<Pubkey, Account, S>
where
    S: std::hash::BuildHasher,
{
    fn load(&self, key: &Pubkey) -> Account {
        self.get(key).cloned().unwrap_or_default()
    }
}

impl AccountLoader for std::collections::BTreeMap<Pubkey, Account> {
    fn load(&self, key: &Pubkey) -> Account {
        self.get(key).cloned().unwrap_or_default()
    }
}
