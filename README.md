# salary_stream

## Project Title
salary_stream

## Project Description
salary_stream is a Soroban smart contract that brings real-time, on-chain salary payments to the Stellar network. Instead of paying employees in monthly lumps, an employer opens a continuous salary stream that accrues value every second; the employee can withdraw their accrued balance at any time without waiting for a pay cycle. The contract is fully self-custodial, requires no custodian, and gives both parties simple, transparent controls (pause, resume, close) for the lifetime of the stream.

## Project Vision
Our vision is to make payroll a primitive on Stellar — as composable and as programmable as any other DeFi building block. By turning salary into a continuously accruing on-chain balance, salary_stream lays the groundwork for a future where workers can stream income, top up DeFi positions from their pay in real time, and prove accruing wages to lenders, landlords, and insurers without intermediaries.

## Key Features
- **Per-second accrual** — salary is computed from a fixed `rate_per_second` and the elapsed ledger time, so balances are always up to date.
- **On-demand withdrawals** — the employee can pull their accrued salary at any moment; no need to wait for a pay-day transaction.
- **Employer controls** — the employer can `pause` and `resume` a stream to handle disputes, leaves, or project boundaries, and `close` it definitively when the engagement ends.
- **Capped duration** — every stream carries an explicit `duration_seconds` cap, so an open stream can never accrue beyond the agreed total.
- **Pure read accessor** — `get_accrued` projects the current unclaimed balance without mutating storage, so it is safe to call from any client or indexer.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** work dApp — see `contracts/salary_stream/src/lib.rs` for the full salary_stream business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CB7HAO4B2AVEJHOT6CURC5LOUQVUVHLW5B3TT66YIBOTKVF7W5KSFEOQ`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/736072748f682201d8ea6c7427ec441228633b896bac3244b82bf036a90eaec1`

## Future Scope
- Integration with a Stellar native asset (XLM) or a Stellar-Classic issued asset so `withdraw` actually transfers value to the employee, not just records an internal balance.
- Switch storage to `persistent` entries with rent bumps so long-running streams survive across many ledgers.
- Support for topping up an existing stream with additional budget or rate, and for partial closes that release part of the remaining balance back to the employer.
- Add event topics (`open`, `withdraw`, `pause`, `resume`, `close`) so off-chain indexers and dashboards can stream live payroll activity.
- Optional delegated withdrawers (e.g. a multisig or a DAO payroll manager) via a per-stream `authorized` allowance list.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `salary_stream` (work)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
