# Source layout each deployed binary records

Every Rust panic compiles in the file path, line and column of the check that can fail, so the deployed binaries record which source files they were built from and where each check sits. A rebuild has to put every check at exactly these positions. Extracted with solopen (`calibrate.panic_locations`) from the captured programs in [lalalune/pumpfun-source](https://github.com/lalalune/pumpfun-source) `evidence/original/`.

## Pump

Program `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`, captured at slot 461,038,878, 10,485,715-byte account payload. 54 panic sites in 15 of its own source files; 10 of those files don't exist at that path in pumpfun-source.

| File | Line:column | In pumpfun-source |
|---|---|---|
| `programs/pump/src/admin_set_idl_authority.rs` | 86:13, 96:37, 101:45 | missing |
| `programs/pump/src/create_v2.rs` | 27:10, 385:29, 426:64, 488:19 | yes |
| `programs/pump/src/creator/distribute_creator_fee_v2.rs` | 342:22 | missing |
| `programs/pump/src/creator/distribute_creator_fees.rs` | 217:1, 327:22 | missing |
| `programs/pump/src/creator/distribute_fee_to_holders.rs` | 114:22, 121:44 | missing |
| `programs/pump/src/creator/set_creator.rs` | 63:31 | missing |
| `programs/pump/src/creator/set_metaplex_creator.rs` | 43:31 | missing |
| `programs/pump/src/extend_account.rs` | 89:57 | missing |
| `programs/pump/src/lib.rs` | 83:1, 84:1, 90:1, 1442:62, 1443:40, 1477:42, 1478:40, 1815:1, 1936:45, 1939:14, 2639:1, 2664:1 | yes |
| `programs/pump/src/migrate.rs` | 659:43 | yes |
| `programs/pump/src/migrate_v2.rs` | 820:43 | yes |
| `programs/pump/src/multi_hop_curve_swap.rs` | 232:32 | missing |
| `programs/pump/src/quote_control/state.rs` | 16:1 | yes |
| `programs/pump/src/sell.rs` | 76:13, 77:13 | missing |
| `programs/pump/src/token_incentives/volume_accumulator.rs` | 8:1, 19:1, 53:14, 78:17, 108:14, 136:14, 164:22, 170:22, 186:22, 192:22, 317:13, 331:14, 346:13, 376:18, 381:18, 390:18, 397:17, 425:14, 429:43, 431:14, 436:28 | missing |

- 2 panic sites in `src/generated/instructions/create_metadata_account_v3.rs` that no crate in the program's Cargo.lock has: a dependency the reconstruction is missing.
- Locked versions confirmed by the binary: anchor-lang 0.31.1, solana-borsh 2.2.1, spl-token-2022 6.0.0.

## PumpSwap

Program `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA`, captured at slot 461,038,886, 10,485,715-byte account payload. 41 panic sites in 10 of its own source files; 8 of those files don't exist at that path in pumpfun-source.

| File | Line:column | In pumpfun-source |
|---|---|---|
| `programs/pump-amm/src/instructions/coin_creator/set_coin_creator.rs` | 77:36 | missing |
| `programs/pump-amm/src/instructions/extend_account.rs` | 96:57 | missing |
| `programs/pump-amm/src/instructions/swap/buy.rs` | 233:1 | missing |
| `programs/pump-amm/src/instructions/swap/multi_hop_swap.rs` | 233:17, 315:28, 344:32, 353:40, 359:34, 363:41, 369:32, 772:20, 773:10, 775:10, 814:45, 815:58 | missing |
| `programs/pump-amm/src/instructions/swap/sell.rs` | 439:13, 440:13 | missing |
| `programs/pump-amm/src/lib.rs` | 22:1, 23:1, 36:1 | yes |
| `programs/pump-amm/src/state/global_config.rs` | 388:45, 391:14 | yes |
| `programs/pump-amm/src/state/volume_accumulator.rs` | 8:1, 25:1, 54:14, 79:17, 109:14, 137:14, 271:13, 285:14, 300:13, 334:18, 339:18, 348:18, 359:17, 387:14, 391:43, 393:14, 398:28 | missing |
| `programs/pump-amm/src/utils/math.rs` | 14:1 | missing |
| `programs/pump-amm/src/utils/token.rs` | 341:36 | missing |

- Locked versions confirmed by the binary: anchor-lang 0.31.1, spl-token-2022 6.0.0.

## Fees

Program `pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ`, captured at slot 461,038,893, 948,944-byte account payload. 32 panic sites in 12 of its own source files; 9 of those files don't exist at that path in pumpfun-source.

| File | Line:column | In pumpfun-source |
|---|---|---|
| `programs/pump-fees/src/common/mod.rs` | 21:48, 21:56, 33:48, 33:56 | missing |
| `programs/pump-fees/src/instructions/admin/update_fee_config.rs` | 7:1, 16:1 | missing |
| `programs/pump-fees/src/instructions/admin/upsert_fee_tiers.rs` | 6:1, 15:1 | missing |
| `programs/pump-fees/src/instructions/fee_sharing/admin_cto_sharing_config.rs` | 53:1 | missing |
| `programs/pump-fees/src/instructions/fee_sharing/claim_social_fee_pda.rs` | 100:1 | missing |
| `programs/pump-fees/src/instructions/fee_sharing/create_fee_sharing_config.rs` | 37:1, 119:1 | yes |
| `programs/pump-fees/src/instructions/fee_sharing/create_social_fee_pda.rs` | 36:1 | missing |
| `programs/pump-fees/src/instructions/fee_sharing/update_fee_shares.rs` | 114:1, 228:55 | missing |
| `programs/pump-fees/src/instructions/fee_sharing/update_fee_shares_v2.rs` | 236:18, 278:55 | missing |
| `programs/pump-fees/src/lib.rs` | 27:1, 28:1, 38:1 | yes |
| `programs/pump-fees/src/state.rs` | 50:1, 67:32, 73:12, 113:1, 135:1, 154:1, 398:19, 398:33, 451:19, 451:33 | yes |
| `pump-fees-math/src/lib.rs` | 28:9, 49:23 | missing |

- Locked versions confirmed by the binary: anchor-lang 0.31.1.

## Mayhem

Program `MAyhSmzXzV1pTf7LsNkrNwkWKTo4ougAJ1PPg47MD4e`, captured at slot 461,038,899, 777,408-byte account payload. 64 panic sites in 9 of its own source files; 9 of those files don't exist at that path in pumpfun-source.

| File | Line:column | In pumpfun-source |
|---|---|---|
| `programs/mayhem-program/src/instructions/admin.rs` | 25:1 | missing |
| `programs/mayhem-program/src/instructions/burn.rs` | 10:1, 77:52, 104:33, 110:33 | missing |
| `programs/mayhem-program/src/instructions/extend_account.rs` | 27:1, 94:57 | missing |
| `programs/mayhem-program/src/instructions/fee_recipients.rs` | 13:1, 20:1, 47:41, 58:32, 207:8, 208:16, 276:84, 284:17, 290:17, 291:17, 295:22, 296:86, 303:17, 306:17, 353:84, 358:37, 362:17, 363:17, 367:22, 368:86, 375:17, 378:17 | missing |
| `programs/mayhem-program/src/instructions/pump_amm_interface.rs` | 403:23, 404:26, 595:23, 596:26 | missing |
| `programs/mayhem-program/src/instructions/pump_interface.rs` | 197:1, 321:23, 322:26, 444:23, 445:26 | missing |
| `programs/mayhem-program/src/instructions/pump_interface_v2.rs` | 393:25, 394:26, 591:25, 592:26 | missing |
| `programs/mayhem-program/src/lib.rs` | 19:1, 20:1 | missing |
| `programs/mayhem-program/src/state.rs` | 12:1, 107:23, 122:23, 321:26, 350:43, 378:89, 379:34, 398:90, 399:34, 424:43, 463:14, 464:34, 490:14, 491:34, 505:1, 531:43, 532:32, 558:41, 562:32, 578:49 | missing |

- `hashbrown` 0.15.4 appears as a registry path, but it is the standard library's own copy (std `HashMap`/`HashSet`), not a Cargo.lock dependency.
- 2 panic sites in `src/account_wrapper.rs` that no crate in the program's Cargo.lock has: a dependency the reconstruction is missing.
- Locked versions confirmed by the binary: anchor-lang 1.0.0, spl-token-2022-interface 2.1.0.
