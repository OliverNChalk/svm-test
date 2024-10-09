use std::collections::HashSet;

pub use litesvm::types::{FailedTransactionMetadata, TransactionMetadata, TransactionResult};
use solana_sdk::account::{Account, AccountSharedData};
use solana_sdk::message::legacy::BUILTIN_PROGRAMS_KEYS;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::sysvar::{Sysvar, SysvarId};
use solana_sdk::transaction::{SanitizedTransaction, VersionedTransaction};
use solana_sdk::{
    bpf_loader, bpf_loader_upgradeable, compute_budget, ed25519_program, native_loader,
    secp256k1_program, sysvar,
};

use crate::spl::SplProgram;
use crate::AccountLoader;

const PRE_LOADED: &[Pubkey] =
    &[ed25519_program::ID, secp256k1_program::ID, sysvar::instructions::ID];

pub struct Svm<L> {
    inner: litesvm::LiteSVM,
    pub loader: L,
}

impl<L> Default for Svm<L>
where
    L: AccountLoader + Default,
{
    fn default() -> Self {
        Svm { inner: Self::inner(), loader: Default::default() }
    }
}

impl<L> Svm<L>
where
    L: AccountLoader,
{
    /* /////////////////////////////////////////////////////////////////////////////
                                        Setup
    ///////////////////////////////////////////////////////////////////////////// */

    fn inner() -> litesvm::LiteSVM {
        litesvm::LiteSVM::default()
            .with_builtins()
            .with_lamports(1_000_000u64.wrapping_mul(10u64.pow(9)))
            .with_sysvars()
            .with_sigverify(true)
            .with_blockhash_check(true)
    }

    pub fn new(loader: L) -> Self {
        Svm { inner: Self::inner(), loader }
    }

    /* /////////////////////////////////////////////////////////////////////////////
                                        Chain
    ///////////////////////////////////////////////////////////////////////////// */

    pub fn blockhash(&self) -> solana_sdk::hash::Hash {
        self.inner.latest_blockhash()
    }

    /* /////////////////////////////////////////////////////////////////////////////
                                        Accounts
    ///////////////////////////////////////////////////////////////////////////// */

    pub fn load_program(&mut self, program_id: Pubkey, program_name: &str) {
        let elf = crate::utils::load_program_elf(program_name);
        self.inner.add_program(&bpf_loader::ID, program_id, &elf);
    }

    #[cfg(feature = "spl")]
    pub fn load_spl_program(&mut self, program: SplProgram) {
        let elf = match program {
            SplProgram::Token => litesvm::spl,
        };
    }

    pub fn get(&self, key: &Pubkey) -> Option<Account> {
        self.inner.get_account(key)
    }

    pub fn get_sysvar<T>(&self) -> T
    where
        T: Sysvar + SysvarId,
    {
        self.inner.get_sysvar()
    }

    pub fn set(&mut self, key: Pubkey, account: Account) {
        self.inner.set_account(key, account).unwrap()
    }

    pub fn set_sysvar<T>(&mut self, sysvar: &T)
    where
        T: Sysvar + SysvarId,
    {
        self.inner.set_sysvar(sysvar)
    }

    /* /////////////////////////////////////////////////////////////////////////////
                                        Transactions
    ///////////////////////////////////////////////////////////////////////////// */

    #[allow(clippy::result_large_err)]
    pub fn simulate_transaction(
        &mut self,
        tx: VersionedTransaction,
    ) -> Result<(TransactionMetadata, Vec<(Pubkey, AccountSharedData)>), FailedTransactionMetadata>
    {
        let tx = self.sanitize_and_load_accounts(tx);

        self.inner
            .simulate_transaction(tx.to_versioned_transaction())
    }

    #[allow(clippy::result_large_err)]
    pub fn execute_transaction(
        &mut self,
        tx: VersionedTransaction,
    ) -> Result<TransactionMetadata, FailedTransactionMetadata> {
        let tx = self.sanitize_and_load_accounts(tx);

        self.inner.send_transaction(tx.to_versioned_transaction())
    }

    fn sanitize_and_load_accounts(&mut self, tx: VersionedTransaction) -> SanitizedTransaction {
        // Load any missing lookup tables.
        for key in tx
            .message
            .address_table_lookups()
            .into_iter()
            .flat_map(|table| table.iter().map(|table| table.account_key))
        {
            if self.inner.get_account(&key).is_none() {
                self.inner.set_account(key, self.loader.load(&key)).unwrap();
            }
        }

        // Resolve transaction.
        let hash = tx.message.hash();
        let sanitized = SanitizedTransaction::try_create(
            tx,
            hash,
            Some(false),
            &self.inner.accounts,
            &HashSet::default(),
        )
        .unwrap();

        // Load any missing accounts.
        for key in sanitized.message().account_keys().iter().filter(|key| {
            !PRE_LOADED.contains(key)
                && !BUILTIN_PROGRAMS_KEYS.contains(key)
                && key != &&compute_budget::ID
        }) {
            // We only go to the loader the first time an account is touched.
            if self.inner.get_account(key).is_some() {
                continue;
            }

            // Programs are a bit special.
            let account = self.loader.load(key);
            match (account.executable, account.owner) {
                (true, bpf_loader::ID | native_loader::ID) => {}
                (true, bpf_loader_upgradeable::ID) => {
                    let exec_key = Pubkey::find_program_address(
                        &[key.as_ref()],
                        &bpf_loader_upgradeable::id(),
                    )
                    .0;
                    let exec_data = self.loader.load(&exec_key);

                    self.inner.set_account(exec_key, exec_data).unwrap();
                }
                (true, _) => {
                    panic!("Unexpected program owner; program={key}; owner={}", account.owner)
                }
                (false, _) => {}
            }

            // Set the account.
            self.inner.set_account(*key, account).unwrap();
        }

        sanitized
    }
}
