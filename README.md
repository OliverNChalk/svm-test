# SVM Test

`svm-test` is a library for:

- Unit testing Solana programs.
- Integration testing Solana programs.
- Integration testing off-chain code (that needs to interact with Solana
  programs).

The goals of the `svm-test` crate are:

- Simplicity
- Productivity

I believe this is best achieved by exposing a minimalist API that only solves
the hard/annoying problems in testing Solana programs. Any user-specific helpers
should be written by the user for themselves. My target user is someone who can
code.

## Setup

1. Add the `svm-test` crate to your project.
2. Setup a submodule for test data at path `/test-data`, where `/` is your
   workspace root.

## Writing Tests

See `crates/svm-test/tests` for how to write tests with `svm-test`.
Additionally, you can read the source code (it's quite short).

## CLI Usage

In the future there may be a helper CLI to enable some more advanced use cases,
for now you can interface with the `Harness` via environment variables:

| Key        | Value | Effect                                                             |
| ---------- | ----- | ------------------------------------------------------------------ |
| TEST_RPC   | `URL` | The harness will overwrite local scenarios with data from this RPC |
| TEST_DEBUG | `ANY` | Setting this variable will enable debug logging                    |

## Typical Workflow

1. Write unit tests.
2. Run `TEST_RPC=<URL> cargo test <your-test-name>`
3. `cd test-data && git add -A && git commit -m "data: added new scenario && git push`
4. `cargo test`

## Contributing

If you wish to contribute changes to the API or fixes please open a PR. Keep in
mind the goals of simplicity, new features should enable use cases, not just
make certain use cases more ergonomic at the expense of simplicity.

## Acknowledgement

`svm-test` uses the fabulous [LiteSVM][0] crate for the core transaction & accounts logic.

[0]: https://github.com/LiteSVM/litesvm
