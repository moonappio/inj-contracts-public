# Moon Injective Contracts

## Setup

```bash
# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install cosmwasm-check utility
cargo install cosmwasm-check
```

## Contracts

| Name                                 | Description      |
| ------------------------------------ | ---------------- |
| [`moon-sale`](contracts/moon-sale)   | Moon token sale  |
| [`moon-claim`](contracts/moon-claim) | Moon token claim |
