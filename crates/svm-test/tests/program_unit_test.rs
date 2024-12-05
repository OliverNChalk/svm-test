//! These examples do not make use of `Harness` and only include local accounts
//! & programs.
use expect_test::expect;
use litesvm::types::SimulatedTransactionInfo;
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use svm_test::svm::DefaultLoader;
use svm_test::utils::{test_payer_keypair, TEST_PAYER};
use svm_test::Svm;

const MEMO_ID: Pubkey = Pubkey::new_from_array([1; 32]);

#[test]
fn memo() {
    // Load our program & give our test payer some funds.
    let mut svm: Svm<DefaultLoader> = Svm::default();
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
    let SimulatedTransactionInfo { meta, post_accounts } = svm.simulate_transaction(tx).unwrap();

    // Assert.
    expect![[r#"
        TransactionMetadata {
            signature: 32qrmjZC3XfzvagJHVozcNWnjW24uBrcdGmUP5r31LstZM3nAsH7cENbuWzW29dyQWLoCTBjBYV6xNrPKS7FJPsG,
            logs: [
                "Program 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi invoke [1]",
                "Program 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi consumed 33 of 200000 compute units",
                "Program 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi success",
            ],
            inner_instructions: [
                [],
            ],
            compute_units_consumed: 33,
            return_data: TransactionReturnData {
                program_id: 4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi,
                data: [],
            },
        }
    "#]].assert_debug_eq(&meta);
    expect![[r#"
        [
            (
                AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9,
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
    .assert_debug_eq(&post_accounts);
}
