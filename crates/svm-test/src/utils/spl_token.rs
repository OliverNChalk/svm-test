use solana_sdk::account::Account;
use solana_sdk::program_option::COption;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::AccountState;

use crate::{AccountLoader, Svm};

pub const WSOL: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub fn mock_ata<L>(svm: &mut Svm<L>, mint: Pubkey, owner: Pubkey, amount: u64) -> Pubkey
where
    L: AccountLoader,
{
    let key = get_associated_token_address(&owner, &mint);
    let account = token(mint, owner, amount);
    svm.set(key, program_account(account));

    key
}

pub fn program_account(account: spl_token::state::Account) -> Account {
    Account {
        owner: spl_token::ID,
        data: super::pack_to_vec(account),
        lamports: Rent::default().minimum_balance(spl_token::state::Account::LEN),
        ..Default::default()
    }
}

pub fn token(mint: Pubkey, owner: Pubkey, amount: u64) -> spl_token::state::Account {
    spl_token::state::Account {
        mint,
        owner,
        amount,
        delegate: COption::None,
        delegated_amount: 0,
        state: AccountState::Initialized,
        is_native: match mint {
            _ if mint == WSOL => COption::Some(0),
            _ => COption::None,
        },
        close_authority: COption::None,
    }
}
