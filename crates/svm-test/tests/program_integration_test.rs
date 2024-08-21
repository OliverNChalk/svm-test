//! These examples make use of `Harness` to fetch & store RPC accounts &
//! programs. This allows for seamless integration testing against already
//! deployed testnet/mainnet accounts.
use expect_test::expect;
use solana_sdk::account::{Account, ReadableAccount};
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use svm_test::utils::{test_payer_keypair, TEST_PAYER};
use svm_test::{Harness, Svm};

const USDC: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

#[test]
fn faucet() {
    // Setup our test payer.
    let mut svm = Svm::new(Harness::get().get_scenario("faucet"));
    svm.set(TEST_PAYER, Account { lamports: 10u64.pow(9), ..Default::default() });

    // Setup our faucet.
    svm.load_program(faucet::ID, "faucet");
    let faucet_usdc =
        svm_test::utils::spl_token::mock_ata(&mut svm, USDC, faucet::SPENDER, 5 * 10u64.pow(6));
    let recipient_usdc = svm_test::utils::spl_token::mock_ata(&mut svm, USDC, TEST_PAYER, 0);

    // Prepare a basic transaction.
    let ixs = [Instruction::new_with_bytes(
        faucet::ID,
        &[],
        vec![
            AccountMeta { pubkey: faucet_usdc, is_signer: false, is_writable: true },
            AccountMeta { pubkey: recipient_usdc, is_signer: false, is_writable: true },
            AccountMeta { pubkey: faucet::SPENDER, is_signer: false, is_writable: false },
            AccountMeta { pubkey: spl_token::ID, is_signer: false, is_writable: false },
        ],
    )];
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&TEST_PAYER),
        &[test_payer_keypair()],
        svm.blockhash(),
    );

    // Simulate (run without updating state).
    let (meta, accounts) = svm.simulate_transaction(tx.into()).unwrap();

    // Assert.
    expect![[r#"
        TransactionMetadata {
            signature: TkYH3gZ16Qy5sXhy4Xu1jU8u5MfVgmeiDhN62FiSaCSewV4D3j7DveN7YhHdAGSKc5D7mU1umyoF5KY951hjTQX,
            logs: [
                "Program 69jHfHKn5N6sw9ZacqFzVVhETGiRkh9LD3q9YrfmAA6v invoke [1]",
                "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA invoke [2]",
                "Program log: Instruction: Transfer",
                "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA consumed 4645 of 197360 compute units",
                "Program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA success",
                "Program 69jHfHKn5N6sw9ZacqFzVVhETGiRkh9LD3q9YrfmAA6v consumed 7429 of 200000 compute units",
                "Program 69jHfHKn5N6sw9ZacqFzVVhETGiRkh9LD3q9YrfmAA6v success",
            ],
            inner_instructions: [
                [
                    InnerInstruction {
                        instruction: CompiledInstruction {
                            program_id_index: 3,
                            accounts: [
                                2,
                                1,
                                4,
                            ],
                            data: [
                                3,
                                64,
                                66,
                                15,
                                0,
                                0,
                                0,
                                0,
                                0,
                            ],
                        },
                        stack_height: 2,
                    },
                ],
            ],
            compute_units_consumed: 7429,
            return_data: TransactionReturnData {
                program_id: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA,
                data: [],
            },
        }
    "#]].assert_debug_eq(&meta);
    expect![[r#"
        [
            (
                5Z6Ay5NEcbg3xhopc522sBCRXQujkTiuDRnHGfQdcnSf,
                Account {
                    lamports: 999995000,
                    data.len: 0,
                    owner: 11111111111111111111111111111111,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                5vUprZVMWkZgRdVKoEFp5fJHdeWvGXJQrGErFNwAysL4,
                Account {
                    lamports: 2039280,
                    data.len: 165,
                    owner: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA,
                    executable: false,
                    rent_epoch: 0,
                    data: c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d6143a72e714401762df66b68c26dfbdf2682aaec9f2474eca4613e424a0fbafd3c,
                },
            ),
            (
                Acf7QyKKBaV4QSR1aBCEAEvPWTreSmqSQokQnQxvic5N,
                Account {
                    lamports: 2039280,
                    data.len: 165,
                    owner: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA,
                    executable: false,
                    rent_epoch: 0,
                    data: c6fa7af3bedbad3a3d65f36aabc97431b1bbe4c2d2f6e0e47ca60203452f5d613550b298afcde8c44442accf805d2639b99c76f1b250c3692caad5ecbb14928b,
                },
            ),
        ]
    "#]].assert_debug_eq(&accounts);

    // TODO: Consider making a helper for this.
    let faucet_before = spl_token::state::Account::unpack(&svm.get(&faucet_usdc).unwrap().data)
        .unwrap()
        .amount;
    let faucet_after = spl_token::state::Account::unpack(
        accounts
            .iter()
            .find(|(key, _)| key == &faucet_usdc)
            .unwrap()
            .1
            .data(),
    )
    .unwrap()
    .amount;
    assert_eq!(faucet_before, 5 * 10u64.pow(6));
    assert_eq!(faucet_after, 4 * 10u64.pow(6));

    let recipient_before =
        spl_token::state::Account::unpack(&svm.get(&recipient_usdc).unwrap().data)
            .unwrap()
            .amount;
    let recipient_after = spl_token::state::Account::unpack(
        accounts
            .iter()
            .find(|(key, _)| key == &recipient_usdc)
            .unwrap()
            .1
            .data(),
    )
    .unwrap()
    .amount;
    assert_eq!(recipient_before, 0);
    assert_eq!(recipient_after, 10u64.pow(6));
}
