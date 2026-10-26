# pumpfun-source

Work toward byte-for-byte rebuilds of pump.fun's four Solana programs (Pump, PumpSwap, Fees and Mayhem) with [solopen](https://github.com/SolanaOpen/SolOpen).

This builds on [lalalune/pumpfun-source](https://github.com/lalalune/pumpfun-source), which captured the deployed programs and reconstructed their source. That repository reports **0 of 4** programs byte-identical after 26 passes. This one applies solopen's measurements to its source: what each deployed binary says it was built from, where the reconstruction differs, and a per-function to-do list. All reconstructed source here is lalalune's; credit for it belongs there.

## Status

| Program | Original's own source files | Missing at that path in lalalune's source | Here |
|---|---:|---:|---|
| Mayhem | 9 | 9 | Original layout, `declare_program!` clients, the `pump-cpi` crate and the AI loop; **71.42%** of the original's code matches; [progress](mayhem/README.md#progress), [to-do list](mayhem/TODO.md) |
| Pump | 15 | 10 | [Analysis](analysis/source-layout.md#pump) |
| PumpSwap | 10 | 8 | [Analysis](analysis/source-layout.md#pumpswap) |
| Fees | 12 | 9 | [Analysis](analysis/source-layout.md#fees) |

"Original's own source files" are the files whose panic sites the deployed binary carries. Every check that can fail compiles in its file path, line and column, so a rebuild matches only when each check sits at exactly the recorded position.

## Findings

- **The source layout is wrong for all four programs.** Each binary names files the reconstruction doesn't have: `sell.rs`, `extend_account.rs` and five `creator/` files for Pump; `instructions/swap/{buy,sell,multi_hop_swap}.rs` for PumpSwap; a separate `pump-fees-math` crate for Fees; a `programs/mayhem-program/` crate for Mayhem. [analysis/source-layout.md](analysis/source-layout.md) lists every file with its line and column positions.
- **Dependency versions mostly fit.** anchor-lang 0.31.1 and spl-token-2022 6.0.0 are confirmed for Pump, PumpSwap and Fees, and anchor-lang 1.0.0 for Mayhem. No locked version contradicts the binaries.
- **Two programs use a crate the reconstruction lacks.** Pump has panic sites in a generated `create_metadata_account_v3.rs`, which matches the layout of a generated Metaplex client. Mayhem has two in a `src/account_wrapper.rs` that no public crate we could find has.
- **Mayhem uses a std `HashMap` or `HashSet`.** Its binary carries `hashbrown` 0.15.4, which is the standard library's own copy, not a Cargo.lock dependency.

## Mayhem

[mayhem/](mayhem) started as lalalune's Mayhem source and now reproduces **71.42%** of the original's code byte-for-byte, up from 25.21%. Built with platform-tools v1.54 and relinked in the original's function order. What got it there, step by step, is in [mayhem/README.md](mayhem/README.md#progress):

- the file layout the binary names;
- Pump and PumpSwap calls through the clients `declare_program!` generates, with IDLs trimmed to the versions Mayhem was built against;
- [crates/pump-cpi](crates/pump-cpi), a reconstruction of the unpublished dependency the original's `src/lib.rs` and `src/account_wrapper.rs` locations point to, including the wrapper that zero-pads older accounts;
- solopen's AI rebuild loop (Claude Opus 5.5), one function at a time, keeping an edit only when that function got closer and nothing that already matched broke.

[mayhem/TODO.md](mayhem/TODO.md) lists every remaining function and the current leads. Anthropic's API declined to work on Mayhem's fee-sweep functions, so the AI loop leaves those as lalalune reconstructed them.

## Reproduce

```sh
git clone https://github.com/SolanaOpen/SolOpen && cd SolOpen
python3 solopen.py fetch MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e
python3 solopen.py try --id MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e \
  --workspace ../pumpfun-source/mayhem --program mayhem_program --package mayhem-program --toolchain 1.54
python3 solopen.py target --run runs/<that run> --original capture/MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e/program.so
python3 solopen.py lockfit --id <PROGRAM_ID> --workspace <workspace> --lock <its Cargo.lock>
```

`analysis/lockfit-*.json` are the raw lockfit reports for the four programs, run against lalalune's commit `0b29240`.

## Layout

| Path | What |
|---|---|
| `mayhem/` | Mayhem workspace in the original layout: source, IDLs, `TODO.md`, `targets/` |
| `crates/pump-cpi/` | Reconstructed dependency for Mayhem's v2 Pump calls (kept outside the workspace so its paths compile as `src/…`, as in the original) |
| `analysis/source-layout.md` | Source files and panic positions each deployed binary records |
| `analysis/lockfit-*.json` | Dependency evidence per program |
