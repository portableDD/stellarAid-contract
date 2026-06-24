# Contract Upgrade Process

The `campaign` contract supports in-place WASM upgrades via the `upgrade` function, which uses Soroban's `env.deployer().update_current_contract_wasm(new_wasm_hash)`. All on-chain state is preserved across upgrades.

## Who Can Upgrade

Only the **admin** address can call `upgrade`. The admin defaults to the `creator` address set during `initialize` and can be rotated using `set_admin`.

## Steps

### 1. Build the new WASM

```bash
cargo build -p campaign --target wasm32v1-none --release
```

The optimized artifact is at:
```
target/wasm32v1-none/release/campaign.wasm
```

### 2. Upload the WASM to the network

Use the Stellar CLI to upload the binary and obtain a hash:

```bash
stellar contract upload \
  --wasm target/wasm32v1-none/release/campaign.wasm \
  --network testnet \
  --source <ADMIN_KEY>
```

The command prints the 32-byte WASM hash (hex-encoded).

### 3. Call `upgrade` on the deployed contract

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <ADMIN_KEY> \
  -- upgrade \
  --new_wasm_hash <WASM_HASH_HEX>
```

### 4. Verify

- The contract emits a `contract_upgraded` event containing the admin address, the new WASM hash, and the upgrade timestamp.
- Call any read-only view (e.g. `get_total_raised`) to confirm the contract responds and state is intact.

## Safety Checklist

- [ ] New WASM has been reviewed and tested on testnet.
- [ ] `set_admin` has been used to confirm the admin key is secure and rotated if needed.
- [ ] Contract is **not** frozen (`unfreeze` if necessary before upgrading on mainnet).
- [ ] State-schema changes (new `DataKey` variants) are **append-only** — never remove or renumber existing keys.
- [ ] Emit a public announcement / changelog entry for the upgrade.
