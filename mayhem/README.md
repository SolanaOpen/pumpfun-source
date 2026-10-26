# Mayhem reconstruction in the original layout

Source for the pump.fun Mayhem program (`MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e`), rearranged into the file layout the deployed binary names, as a starting point for a byte-identical rebuild with [solopen](https://github.com/SolanaOpen/SolOpen).

The code started as lalalune's reconstruction from [pumpfun-source](https://github.com/lalalune/pumpfun-source) (`programs/mayhem`, commit `0b29240`). The first step changed no logic, only the layout:

| Before | Now (the paths the binary's panic messages name) |
|---|---|
| crate `mayhem-recovered` in `programs/mayhem` | crate `mayhem-program` in `programs/mayhem-program`, under a workspace root |
| 24 handlers inline in `lib.rs` | `lib.rs` only dispatches; handlers live in `instructions/` |
| `buy.rs`, `sell.rs` | `instructions/pump_interface.rs` |
| `buy_v2.rs`, `sell_v2.rs` | `instructions/pump_interface_v2.rs` |
| `buy_pump_swap.rs`, `sell_pump_swap.rs`, `pump_swap_types.rs` | `instructions/pump_amm_interface.rs` |
| admin, withdraw and global-params handlers | `instructions/admin.rs` |
| fee recipient and sweep handlers | `instructions/fee_recipients.rs` |
| extend handler | `instructions/extend_account.rs` |
| `GlobalParams` | `state.rs` |

Each old file is kept whole as an inline module (`pump_interface::buy`, …), so its contents are unchanged.

## Progress

Share of the original's code reproduced byte-for-byte, after relinking in the original's function order:

| Step | Coverage |
|---|---:|
| lalalune's source as published | 25.21% |
| Moved into the original's file layout (no logic changed) | 25.21% |
| Pump and PumpSwap calls through `declare_program!(pump)` / `declare_program!(pump_amm)` on `lib.rs` lines 19 and 20, with the IDLs in `idls/` (Pump's `buy` without the later `partial_fill` argument) | 32.51% |
| PumpSwap `GlobalConfig` and Pump `BondingCurve` as the generated types, IDLs trimmed to Mayhem-era layouts (418 and 82 bytes) | 32.79% |
| Pump v2 calls through [`crates/pump-cpi`](../crates/pump-cpi), a crate outside the workspace with `declare_program!(pump)` at `src/lib.rs:8:5` | 37.92% |
| AI rebuild loop (Claude Opus 5.5 through solopen `rebuild`): 92 model calls, 22 edits kept | 54.32% |
| `pump-cpi`'s `AccountWrapper` zero-padding older accounts, and its `declare_program!(pump_amm)` at line 15: PumpSwap `Pool` and Pump's 141-byte bonding curve read through it | 59.53% |
| AI rebuild loop, segment 4: 102 model calls, 21 edits kept so far | 71.62% |

[TODO.md](TODO.md) lists every function still to fix, the original's panic sites file by file, and the current leads. `targets/` holds an assembly diff per function.

## Build and score

```sh
python3 solopen.py try --id MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e \
  --workspace path/to/this --program mayhem_program --package mayhem-program --toolchain 1.54
python3 solopen.py target --run runs/<that run> --original capture/MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e/program.so
```

platform-tools v1.54 is pumpfun-source's measured choice; solopen's `toolchain auto` rates v1.54 to v1.56 a close call for this program.
