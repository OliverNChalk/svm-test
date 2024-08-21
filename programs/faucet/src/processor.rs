use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::program::invoke_signed;
use solana_program::pubkey::Pubkey;

pub fn process_instruction(_: &Pubkey, accounts: &[AccountInfo], _: &[u8]) -> ProgramResult {
    let mut accounts = accounts.iter();
    let from = accounts.next().unwrap();
    let to = accounts.next().unwrap();
    let spender = accounts.next().unwrap();

    // CPI a transfer of 1e6 tokens.
    let ix = spl_token::instruction::transfer(
        &spl_token::ID,
        from.key,
        to.key,
        &crate::SPENDER,
        &[],
        10u64.pow(6),
    )
    .unwrap();
    let accounts = vec![from.clone(), to.clone(), spender.clone()];
    invoke_signed(&ix, &accounts, crate::SPENDER_SEEDS)?;

    Ok(())
}
