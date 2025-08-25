# How TWAP Price is Retrieved On-Chain

## Overview

- The reclaim price is set on-chain in a decentralized manner.
- It must reflect historical AMM trading prices, such as the Time-Weighted Average Price (TWAP).

## Considerations

- We design the model to showcase how to get the twap price, using SOL/USDC pair for example.
- We may have to create an indexer for the custom tokens we mint and make a price per pull update (Create a pyth price account)
- Other customizations will come in as we iterate, we may find better ways to track the twap per token fragments

## TWAP Price Retrieval Using Pyth

- We use the Pyth oracle to fetch price data on-chain.
- The logic for retrieving and using the TWAP price is implemented in `src/processor.rs`.
- The processor reads the Pyth price account, validates the data, and computes the TWAP as required by the protocol.
- After running the tests, the on-chain transaction logs—including TWAP price retrieval—are available for inspection. You can view detailed logs and results on Solscan by following the transaction link, for example: https://solscan.io/tx/4QmNZfmo4ppcQZwDaLE42rEgf96hTVVYN1eoiXZGyvDcnnKZuXEPfDdDpag8MS6FkVWbG1LUzZTsTzZJbZqXAA3F?cluster=devnet

## Key Implementation Details

- The program interacts with the Pyth price feed account.
- TWAP calculation and validation logic are handled in the processor.
- Error handling and account checks are performed to ensure data integrity.

## Deployment and Testing

- Use the provided `devops.sh` script to deploy the program to the Solana cluster.
- shellexpand = "3.1.1" is needed in dependencies for only test
- Run `test.sh` to execute the test suite, including tests for TWAP price retrieval and validation.
- Example commands:
  - `./devops.sh deploy` – Deploys the program.
  - `./devops.sh run-tests` – Runs all tests.

## References

- See `src/processor.rs` for the TWAP and Pyth integration logic.
- For deployment and testing automation, refer to `devops.sh`
