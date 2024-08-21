#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
pub mod processor;

pub const SPENDER: solana_program::pubkey::Pubkey =
    solana_program::pubkey!("4b7ygvbo9tfTgXHVp3V97j8iWVxubejddLDWBPhZA78v");
const SPENDER_SEEDS: &[&[&[u8]]] = &[&[b"spender", &[SPENDER_BUMP]]];
const SPENDER_BUMP: u8 = 252;

solana_program::declare_id!("69jHfHKn5N6sw9ZacqFzVVhETGiRkh9LD3q9YrfmAA6v");

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn baron_pda() {
        assert_eq!(
            solana_program::pubkey::Pubkey::find_program_address(&[b"spender"], &crate::ID),
            (SPENDER, SPENDER_BUMP)
        );
    }
}
