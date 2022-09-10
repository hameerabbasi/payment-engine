# Payment Processor
Simple payment processor.

## Structure
### Input Handling
We use [`serde`](https://crates.io/crates/serde) and [`csv`](https://crates.io/crates/csv) to do the file reading, and [`clap`](https://crates.io/crates/clap) to parse the command-line arguments.
### Transactions
The file [`transaction.rs`](src/transaction.rs) stores the logic that allows `serde` to read the transactions in the expected format. We perform validation using the `serde(try_from = "...")` feature to make sure that the transactions we construct are valid by construction.

### Program Flow
The file [`state.rs`](src/state.rs), specifically the function `CurrentState::add` stores the logic that is used for the bulk of the processing. The fields refer to:

* The past history of transactions in `CurrentState::transactions`
  * This is necessary to store because we need to retrieve the amount associated with a transaction during disputes, resolves, or chargebacks
* The list of disputes in `CurrentState::disputes`
  * Mostly used for validating if a dispute already exists, so as to prevent duplicates
* Client data
  * Used to maintain client status.

### Error handling
There is religious error handling using the crate [`thiserror`](https://crates.io/crates/thiserror) and its derive feature, so other users of the code can tell error types apart. Error types are defined in [`errors.rs`](src/errors.rs), and most checking apart from initial validity checks are done in `CurrentState::check_*` functions in the [`state.rs`](src/state.rs) file. Parts of this leverage the type system for better checks.

The function `CurrentState::read_all`, prints errors to `stderr`, but it does *not* abort a program. An analogy with an ATM (or a server) is that one wouldn't want it to crash on an invalid operation, but it *should* report the error appropriately. To see the real output, please redirect `stderr` to a file (or `/dev/null` to ignore all errors).

## TODO
- [ ] While the program only stores necessary information, this can still overflow RAM. Writing to a database would help.
