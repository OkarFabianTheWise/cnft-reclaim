# On-Chain TWAP Oracle Program

## Overview

This Solana program implements a simple on-chain oracle for Time-Weighted Average Price (TWAP) using a circular buffer of price observations. It is designed to track the average price of a token pair (e.g., SOL/USDC) over a configurable time window, using on-chain AMM pool data from raydium.

## How It Works

- The program maintains a fixed-size buffer (default: 3) of price observations, each spaced by a minimum interval (default: 5 minutes).
- Each observation records the price at a specific timestamp, fetched from the AMM pool vaults.
- New observations are only recorded if enough time has passed since the last one, ensuring the buffer covers a rolling window (e.g., 15 minutes for 3 observations × 5 minutes).
- You can resize this to accommodate more observations, and reduce time intervals
- The buffer is implemented as a circular array, so the oldest observation is overwritten as new ones are added.
- The TWAP is calculated on-chain using the stored observations.

## How to connect the price update

Each time the fragments move(swaps), we execute the price update; We only save the snapshot of the price as observations in the program, this way we don't outrightly depend entirely on the pool for price update eluding manipulation.

## Key Implementation Details

- The buffer size and observation interval are set by the constants `OBSERVATION_COUNT` and `OBSERVATION_INTERVAL` in `src/lib.rs`.
- Price is calculated from the AMM pool's vault token balances.
- All logic for updating observations and calculating TWAP is on-chain and permissionless.
- The program can be extended to support more pairs or different intervals by changing the constants.
- Also the burn mechanism in reclaim method is meant to retrieve the fractions for compensation.
- We can now join the pieces together after review

## Deployment and Testing

- Use the provided `devops.sh` script to deploy the program to a Solana cluster.
- Run `test.sh` to execute the test suite, including TWAP logic.
- shellexpand = "3.1.1" is needed to get your keypair from cli
- Example commands:
  - `./devops.sh deploy` – Deploys the program.
  - `./devops.sh run-tests` – Runs all tests.

## Example

If `OBSERVATION_COUNT = 3` and `OBSERVATION_INTERVAL = 300` (5 minutes), the buffer will store 3 prices over 15 minutes, e.g.:

    [ (12:00, 100), (12:05, 99.8), (12:10, 101) ]

The TWAP is then calculated from these values.

## References

- See `src/lib.rs` for the TWAP buffer and logic.
- For deployment and testing automation, refer to `devops.sh`.
