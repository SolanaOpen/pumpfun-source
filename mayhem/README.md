# Mayhem reconstruction in the original layout

Source for the pump.fun Mayhem program (`MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e`), rearranged into the file layout the deployed binary names, as a starting point for a byte-identical rebuild with [solopen](https://github.com/SolanaOpen/SolOpen).

The code is lalalune's reconstruction from [pumpfun-source](https://github.com/lalalune/pumpfun-source) (`programs/mayhem`, commit `0b29240`). No logic was changed. Only the layout moved:

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

## Where it stands

| | Before | Now |
|---|---:|---:|
| Original code reproduced byte-for-byte | 25.21% | 25.21% |
| Source files named that the original doesn't have | 7 | 0 |
| Differing bytes after relinking in the original order | 398,929 | 398,237 |

Coverage did not move because no code changed; the layout is what the remaining work needs. [TODO.md](TODO.md) lists every function still to fix, the original's panic sites file by file, and the leads found so far. `targets/` holds an assembly diff per function.

## Build and score

```sh
python3 solopen.py try --id MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e \
  --workspace path/to/this --program mayhem_program --package mayhem-program --toolchain 1.54
python3 solopen.py target --run runs/<that run> --original capture/MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e/program.so
```

platform-tools v1.54 is pumpfun-source's measured choice; solopen's `toolchain auto` rates v1.54 to v1.56 a close call for this program.
