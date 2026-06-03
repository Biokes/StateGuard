# StateGuard

> Automated, decentralized State Keeper Network and tooling for the Soroban smart contract platform.
StateGuard monitors active contract state, dynamically calculates Time-To-Live (TTL) storage metrics, and triggers programmatic extensions or restorations of archived ledger entries to prevent perm[...]

---

## Why This Project Exists

Unlike legacy EVM environments where data lives on-chain indefinitely, Soroban implements a strict **State Expiration and Rent model** to prevent state bloat. This paradigm introduces critical dev[...]

- **Temporary storage entries** expire and are permanently deleted after their TTL expires, with no possible recovery pathway.
- **Expired persistent and instance storage entries** are archived, causing critical dApp features (such as user balance tracking) to "brick" until explicitly restored.
- **Manual state management** requires developers to construct complex, custom extension logic within every contract function, increasing gas fees and user overhead.
- **System operators** lack unified tooling to easily track, budget, and trigger state extensions for dormant or low-activity contracts.

StateGuard solves these challenges through an automated keeper network and unified developer tooling.

---

## Repository Structure

```
├── contracts/
│   └── stateguard-core/    # Rust-based Soroban smart contracts implementing
│                           # automated keeper vaults, deposit escrows, and execution triggers
├── indexer/
│   └── zephyr-monitor/     # Custom Mercury ZephyrVM program compiling to WASM to index,
│                           # filter, and stream live ledger TTL data to the dashboard and keeper bots
└── docs/                   # System documentation, including step-by-step restoration
                            # setup and gas optimization configurations
```

---

## Quick Start

### Prerequisites

- Git, Node.js 22+, npm 10+
- Rust stable + Cargo
- Stellar CLI configured for Testnet or Mainnet

### Verify Toolchain

```bash
cargo --version
stellar --version
```

### Build and Deploy Keeper Contract

```bash
cd contracts/stateguard-core
cargo build --target wasm32-unknown-unknown --release
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stateguard_core.wasm \
  --network testnet \
  --source-account alice
```

### Run Zephyr Indexer Node

To ingest live ledger metadata and monitor expiring TTL footprints:

```bash
cd indexer/zephyr-monitor
cargo test --all-targets
cargo run --bin zephyr-indexer-agent --config config.toml
```

---

## Security

To report a vulnerability, see our [Security Policy](.github/SECURITY.md).

StateGuard treats all transaction execution pathways as highly sensitive. Key security requirements:

- Keeper private keys **must** be stored in secure KMS hardware.
- Contracts **must** verify the caller's authorization before executing `extend_instance_ttl` or `extend_code_ttl` methods on behalf of administrative addresses.

---

## License

This project is licensed under the MIT License — see the [LICENSE](.github/LICENSE)  file for details.


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

- Ensure you have the prerequisites installed (see "Verify Toolchain" above).
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
# Option A: rebase
git checkout -b my-feature-branch
git rebase main
# Option B: merge
# git checkout -b my-feature-branch
# git merge main
```

Push your feature branch to your fork:

```bash
git push origin my-feature-branch
```

5. Make changes & run tests

- Implement your changes on your feature branch.
- Run the relevant build and tests:

```bash
# Build contracts
cd contracts/stateguard-core
cargo build --target wasm32-unknown-unknown --release
# Run unit/integration tests
cargo test
```

6. Create a Pull Request

- Go to your fork on GitHub and click "Compare & pull request" for your branch.
- Target branch: `main` of `Biokes/StateGuard`.
- Fill the PR template (title, description, what you changed, why, how to test).
- Link any related issues.

7. Syncing your fork after the PR

If the upstream main branch has advanced and you need to update your branch:

```bash
git fetch upstream
git checkout main
git pull upstream main
git checkout my-feature-branch
git rebase main
# or: git merge main
git push --force-with-lease origin my-feature-branch
```

8. After review

- Address reviewer comments in the same branch and push updates.
- When CI passes and maintainers approve, a maintainer will merge your PR.

Contribution guidelines & checklist

- Fork and work on feature branches (do not commit directly to `main`).
- Include tests where applicable and ensure all tests pass.
- Follow code formatting and linting rules used in the repo.
- Keep changes scoped to a single concern per PR when possible.

Questions or help

If you need help, open an issue describing the problem or join discussions in the repository.

---

