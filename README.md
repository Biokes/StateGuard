# StateGuard

> Automated, decentralized State Keeper Network and tooling for the Soroban smart contract platform.

StateGuard monitors active contract state, dynamically calculates Time-To-Live (TTL) storage metrics, and triggers programmatic extensions or restorations of archived ledger entries to prevent permanent data deletion.

---

## How It Works

StateGuard is built using a strict **Hexagonal Architecture** (Ports and Adapters) designed to enforce extreme reliability. It operates across multiple independent layers to ensure that a drop in network connectivity or an RPC failure never results in a smart contract unexpectedly expiring.

1. **The ZephyrVM Indexer Pipeline**: StateGuard connects to the Mercury ZephyrVM to consume real-time streams of ledger closures. This ensures sub-5-second latency when detecting expiring state, completely bypassing the rate limits and polling overhead of traditional Stellar RPC nodes.
2. **The Inner Core Domain**: At the heart of the system is a pure, zero-dependency Rust library (`stateguard-core`). It maintains local tracking of your **Keeper Vault** (the XLM escrow used to pay network fees) and calculates probabilistic extension thresholds. It ensures your Vault balance mathematically cannot drop below the minimum required reserves to protect your registered contracts.
3. **The Tokio Actor Daemon**: The `stateguard-daemon` runs in the background using isolated async Actors. One actor listens for ledgers, one evaluates TTL boundaries, and one constructs XDR transactions. This MPSC-channel architecture guarantees that a hung network request will never block the system from analyzing new ledger events.
4. **Trustless Execution**: Before broadcasting any transaction, StateGuard simulates the Soroban footprint to determine the exact read/write costs. It wraps this inside a `ExtendFootprintTTLOp` and dynamically scales base fee bids to ensure inclusion even during network congestion.

### Contextual Importance in the Ecosystem

Unlike legacy EVM environments where data lives on-chain indefinitely, Soroban implements a strict **State Expiration and Rent model** to prevent state bloat. This paradigm introduces critical developer challenges:

- **Temporary storage entries** expire and are permanently deleted after their TTL expires, with no possible recovery pathway.
- **Expired persistent and instance storage entries** are archived, causing critical dApp features (such as user balance tracking) to "brick" until explicitly restored.
- **Manual state management** requires developers to construct complex, custom extension logic within every contract function, increasing gas fees and user overhead.

StateGuard abstracts this entirely. By running StateGuard, DeFi protocols, AMMs, and Oracle networks can focus purely on business logic. StateGuard operates as a silent guardian in the background, guaranteeing that your users never experience "bricked" contracts due to state rent expiration.

---

## Repository Architecture Roadmap

This project is structured as a multi-crate Rust workspace to separate the core invariants from external infrastructure. 

```
StateGuard/
├── stateguard-core/        # Pure Rust domain models, tokenomics, and TTL calculus
├── stateguard-adapters/    # Infrastructure adapters (Stellar SDK, Mercury ZephyrVM, SQLite)
├── stateguard-daemon/      # Long-running Tokio background service orchestrating Actors
├── stateguard-cli/         # Interactive operator CLI for Vault and secret management
├── stateguard-ui/          # Real-time React/Vite operational dashboard
└── stateguard-contracts/   # Trustless Soroban smart contracts acting as Vault escrows
```

---

## Contributing

Thanks for your interest in contributing to StateGuard! We welcome contributions of all kinds: bug reports, documentation improvements, tests, and code changes. To contribute, follow these steps:

1. Fork the repository
   - Click the "Fork" button at the top-right of the repository page to create your own copy under your GitHub account.

2. Clone your fork locally

```bash
git clone https://github.com/<your-username>/StateGuard.git
cd StateGuard
```

3. Verify installation and toolchain

- Ensure you have the prerequisites installed.
- Run the verification commands:

```bash
cargo --version
stellar --version
```

4. Keep your fork up-to-date with main

Add the upstream remote (only once):

```bash
git remote add upstream https://github.com/Biokes/StateGuard.git
```

Before starting work, fetch and rebase (or merge) the latest main branch:

```bash
git fetch upstream
git checkout main
git pull upstream main
git checkout -b my-feature-branch
git rebase main
```

Push your feature branch to your fork:

```bash
git push origin my-feature-branch
```

5. Make changes & run tests

- Implement your changes on your feature branch.
- Run the relevant build and tests:

```bash
cargo check
cargo test
```

6. Create a Pull Request

- Go to your fork on GitHub and click "Compare & pull request" for your branch.
- Target branch: `main` of `Biokes/StateGuard`.
- Fill the PR template (title, description, what you changed, why, how to test).

7. Contribution guidelines & checklist

- Fork and work on feature branches (do not commit directly to `main`).
- Include tests where applicable and ensure all tests pass.
- Follow code formatting (`cargo fmt`) and linting (`cargo clippy`) rules used in the repo.
- Keep changes scoped to a single concern per PR when possible.

---

### Prerequisites

- Git, Node.js 22+, npm 10+
- Rust stable + Cargo
- Stellar CLI configured for Testnet or Mainnet

### Verify Toolchain

```bash
cargo --version
stellar --version
```

---

## Security

To report a vulnerability, chat me on [Telegram](https://t.me/biokes_art).

StateGuard treats all transaction execution pathways as highly sensitive. Key security requirements:

- Keeper private keys **must** be stored in secure KMS hardware or native OS keyrings.
- Contracts **must** verify the caller's authorization before executing `extend_instance_ttl` or `extend_code_ttl` methods on behalf of administrative addresses.

---

## License

This project is licensed under the MIT License — see the `LICENSE` file for details.
