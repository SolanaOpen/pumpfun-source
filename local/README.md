# Running the AI rebuild loops on your own machine

One container runs solopen's AI rebuild loop on all four pump.fun programs at once and pushes every verified gain back to this repo. It is built for Linux x86-64, the platform the deployed programs were compiled on, because only that toolchain is established to produce byte-identical output. On Windows it runs under WSL2 at native speed; on an Apple-silicon Mac it runs under Docker with Rosetta translation.

The loop builds and runs code a model wrote. Keep it inside the container, on a machine with no wallet keys or other secrets you would mind it touching.

## What it does

For each program it fetches the deployed binary once, then repeatedly: picks one function that does not match the original yet, asks the model for an edit, builds and scores it, and keeps the edit only if that function got closer and nothing that already matched broke. Each kept edit is verified against the exact source that was scored, then committed and pushed. A function the endpoint declines is left as it is and listed in that program's `TODO.md`; the loop moves on.

## You provide two secrets

Both are passed in as environment variables and never written into any file.

- `SOLOPEN_AI_API_KEY` — your Anthropic API key.
- `GITHUB_TOKEN` — a GitHub token with push access to the repo. A fine-grained token scoped to `SolanaOpen/pumpfun-source` with Contents: read and write is enough.

## Windows (RTX 5090 / 9800X3D)

1. **Install WSL2.** In an Administrator PowerShell: `wsl --install`. Reboot, then open "Ubuntu" from the Start menu and set a username.
2. **Install Docker Desktop**, and in its Settings → Resources → WSL integration, enable your Ubuntu distro. (Or install Docker directly inside Ubuntu.)
3. **Work inside the Linux home**, not `/mnt/c`. Builds on the Windows drive are several times slower. In the Ubuntu terminal:
   ```sh
   git clone https://github.com/SolanaOpen/pumpfun-source ~/pumpfun-source
   cd ~/pumpfun-source/local
   ```
4. **Build the image** (once):
   ```sh
   docker build -t solopen-runner .
   ```
5. **Run it:**
   ```sh
   docker run --rm -it \
     -e SOLOPEN_AI_API_KEY="sk-ant-..." \
     -e GITHUB_TOKEN="github_pat_..." \
     -v solopen-work:/work \
     solopen-runner
   ```
   The `-v solopen-work:/work` keeps the captures, build caches and clone between runs, so stopping and restarting picks up where it left off. The 9800X3D has enough cores to run all four programs at once; each loop still waits on the model between edits, so CPU is not the limit for a single program.

## macOS (M-series)

Same as above, but install Docker Desktop (or OrbStack) for Mac and build the image with Rosetta, which the `--platform=linux/amd64` line in the Dockerfile selects automatically. Enable Rosetta in Docker Desktop → Settings → General. It runs slower than on the Windows box but works.

## Watching and stopping

Logs stream to the terminal and are also written under `/work/logs` inside the container. To watch one program from another terminal:
```sh
docker exec -it <container> tail -f /work/logs/mayhem.log
docker exec -it <container> tail -f /work/logs/mayhem.push.log
```
Ctrl-C in the run terminal stops every loop. Nothing half-tested is pushed: a push happens only after a build has scored the edit.

## Options

Set these as extra `-e NAME=value` flags:

| Variable | Default | Meaning |
|---|---|---|
| `PROGRAMS` | all four | subset to run, e.g. `-e PROGRAMS="mayhem pump-fees"` |
| `MODEL` | `claude-opus-5-5` | model name sent to the endpoint |
| `BASE_URL` | `https://api.anthropic.com/v1` | the OpenAI-compatible endpoint |
| `ITERS` | `150` | model calls per segment per program |
| `SEGMENTS` | `20` | segments per program before it stops on its own |
| `RPC_URL` | public mainnet RPC | Solana RPC for the one-time capture; a private RPC is faster and avoids rate limits |

## Cost

Every edit, kept or not, is one model call. A segment is up to `ITERS` calls, and each call sends a function's assembly diff plus its source (tens of thousands of tokens in, a few thousand out). Watch your spend in the provider's console, and start with `-e SEGMENTS=1` on one program to see the rate before running all four.

## What it cannot finish

The reconstruction is byte-for-byte or nothing, and some functions may never match from the public binary alone, for example code that depended on source that was never published. Functions the endpoint declines stay as the original author left them. None of this is worked around; the gaps stay visible in each program's `TODO.md`.
