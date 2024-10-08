use std::sync::OnceLock;

use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::SeedDerivable;

pub const TEST_PAYER: Pubkey = pubkey!("AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9");
static TEST_PAYER_KEYPAIR: OnceLock<Keypair> = OnceLock::new();
pub fn test_payer_keypair() -> &'static Keypair {
    TEST_PAYER_KEYPAIR.get_or_init(|| Keypair::from_seed(&[1; 32]).unwrap())
}

#[cfg(test)]
mod tests {
    use solana_sdk::signer::Signer;

    use super::*;

    #[test]
    fn test_payer() {
        assert_eq!(test_payer_keypair().pubkey(), TEST_PAYER);
    }
}
