# StateGuard
An automated, decentralized State Keeper Network and automated tooling built on the Soroban smart contract platform. It monitors active contract state, dynamically calculates Time-To-Live (TTL) storage metrics, and triggers programmatic extensions or restorations of archived ledger entries to prevent permanent data deletion.

# StateGuard

> Automated, decentralized State Keeper Network and tooling for the Soroban smart contract platform.

StateGuard monitors active contract state, dynamically calculates Time-To-Live (TTL) storage metrics, and triggers programmatic extensions or restorations of archived ledger entries to prevent permanent data deletion.

---

## Why This Project Exists

Unlike legacy EVM environments where data lives on-chain indefinitely, Soroban implements a strict **State Expiration and Rent model** to prevent state bloat. This paradigm introduces critical developer challenges:

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

<!-- Add license here -->
