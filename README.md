# tx_engine
## Overview
`tx_engine` is a Rust-based program for processing client tranasctions being read in from a csv file. The transactions have a certain specification and one of the assumptions
of the engine is that there are some guarantees about the data. For instance, one type of transaction, a dispute, have an associated amount. On the other hand, a deposit does have an associated amount, and the engine assumes that no deposit record missing an amount will ever be read.

Examples of input exist in the `tests` directory. The tests are integration only given the nature of the program and they are quick and dirty. Many intermediate stages
of testing have been performed and the files in the `tests` directory represent the final state of changing tests. The data isn't so clean but it covers both standard
and interesting cases well. The binary can be run against any particular file using the following command:

```cargo run -- $TEST_FILE_PATH```

It can also be tested with the following command:

```cargo test -- --test-threads=1```

> :warning: **It is important that that the binary not be tested with more than one thread (i.e. with `cargo test` alone). Use the option `--test-threads=1` at all times. Note that `cargo testing` is aliased to `cargo test-threads=1`.**

`tx_engine` is stable as a result of strict, often redundant checking on transactions (although not every possible redundancy is covered). The program is also secure
given Rust's security guarantees, the exact needs of the program (e.g. a lack of opportunities for data races), and the fact that all dependencies were chosen
because they are solid, popular crates.

## Logging
`tx_engine` has logging provided by the `tracing_config` and `tracing` crates. Data points about all kinds of transactions, but usually only in "error" cases, are logged and placed into a log file in the root of the repo. These are mostly at the level of `WARN` and `INFO`, but `DEBUG` can also provide much more information. This can be done by setting `level = "debug"` in the `filter.root` section of the `tracing.toml` file in the root of the repo. Also, the logs rotate daily and up to seven log files are kept, and these are configuration options also to be found in `tracing.toml`.


## Many CSVs
What about many CSVs? What if the data from them were streaming through on many different TCP connections? This would be a good case for a call to `mpsc::channel()`. Having many threads, each with a `Sender` and taking every CSV record one-by-one would be able to send valid `tx_engine::Tx` instances over the channel to a single worker thread with a `Receiver`, which would then process each transaction in the way `tx_engine` currently does. This fan-in approach, while efficient for many sources of streaming data, would have to be handled carefully. With only a single thread updating the data stores, there is little concern about synchronizing operations in memory, but the data has to have some guarantees. Without guarantees, there may be concurrent transactions on a client account and the end result will not be what is desired. For example, if CSV 1 has a transaction encoding a deposit to client 1 (starting balance $0) of $5000 while CSV 2 has a transaction encoding a withdrawal of $3000 from client 1's account, what if the withdrawal is processed before the deposit? In this case, the withdrawal would be rejected, even if in a "real-world" representation the deposit preceded the withdrawal. That may not be the case with two or more different CSVs. An easy solution would be to partition the data across CSVs based on the clients (e.g. client 1's transactions appear chronologically sorted in CSV 1 and nowhere else); however, in any case, decisions have to be made about the data organization and its guarantees.



## The other branch
There is a second branch called `consumer-clients` which models the clients as consumers (as opposed to merchants, which is what `master` does). The difference is in the dispute, resolve, and chargeback operations. This branch models the situation as follows:

* A customer initiates a dispute and receives a temporary credit to the account which is not made available immediately (the dispute is on a withdrawal / charge).

* If the dispute is resolved, the credit is reversed.

* If the dispute ends in a chargeback, the credit is granted as available to the client and the client's account is locked.

* If an account is locked, the client will not be able to make withdrawals, initiate disputes, or receive a chargeback. Existing disputes can be resolved and deposits can still be made (this is the inverse of `master`, where withdrawals and disputes can still proceed)..




