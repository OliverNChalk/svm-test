//! These examples do not make use of `Harness` and only include local accounts
//! & programs.
use std::collections::HashMap;

use expect_test::expect;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use svm_test::utils::{test_payer_keypair, TEST_PAYER};
use svm_test::Svm;

const MEMO_ID: Pubkey = Pubkey::new_from_array([1; 32]);

#[test]
fn memo() {
    // Load our program & give our test payer some funds.
    let mut svm = Svm::new(HashMap::default());
    svm.load_program(MEMO_ID, "memo");
    svm.set(TEST_PAYER, Account { lamports: 10u64.pow(9), ..Default::default() });

    // Prepare a basic transaction.
    let ixs = [Instruction::new_with_bytes(MEMO_ID, b"hello", vec![])];
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
            signature: g7Cd7AET73a2vCsjgaoP49tTwvNJ1Hxqih3NFvG5mHZvWY9rMfr4s11GStNaWAdHJkFEKLdqDh88bVZiEtHRBFP,
            logs: [
                "Program 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi invoke [1]",
                "Program 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi consumed 36 of 200000 compute units",
                "Program 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi success",
            ],
            inner_instructions: [
                [],
            ],
            compute_units_consumed: 36,
            return_data: TransactionReturnData {
                program_id: 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi,
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
        ]
    "#]]
    .assert_debug_eq(&accounts);
}
