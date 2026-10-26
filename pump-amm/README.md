# PumpSwap reconstruction

Source for pump.fun's PumpSwap program (`pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`), starting from lalalune's reconstruction in [pumpfun-source](https://github.com/lalalune/pumpfun-source) (`programs/pump-amm`, commit `0b29240`), placed under a workspace root so paths compile as `programs/pump-amm/src/…`, which is what the deployed binary records. Built with platform-tools v1.44.

lalalune's source patched `solana-pubkey` 2.4.0 with a hand-edited copy so a bs58 formatter would match the original. solopen refuses dependency patches by design (a match reached by editing a dependency proves nothing about the program), so this workspace uses the published crate. Finding which published `solana-pubkey` release carries that formatter is an open lead; `lockfit --resolve-ties` is the tool for it.

## Progress

| Step | Coverage |
|---|---:|
| lalalune's source as published, in a workspace root | 36.13% |

[TODO.md](TODO.md) and `targets/` appear after the first AI loop segment runs ([local/](../local)).
