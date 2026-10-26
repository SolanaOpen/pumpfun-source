#!/usr/bin/env bash
# Runs solopen's AI rebuild loop on every program in programs.json, in parallel,
# pushing each kept edit to the repo once it is verified against the scored build.
#
# Required environment:
#   SOLOPEN_AI_API_KEY   Anthropic API key (never written to disk)
#   GITHUB_TOKEN         token with push access to the repo (never written to disk)
# Optional:
#   PROGRAMS   space-separated subset, e.g. "mayhem pump-fees"   (default: all)
#   MODEL      default claude-opus-5-5
#   BASE_URL   default https://api.anthropic.com/v1
#   ITERS      model calls per segment per program (default 150)
#   SEGMENTS   segments per program (default 20)
#   REPO_URL   default https://github.com/SolanaOpen/pumpfun-source
#   RPC_URL    Solana RPC for the initial capture (default mainnet-beta public RPC)
#
# Declined prompts: when the endpoint declines a function (content_filter), solopen
# leaves that function as it is and moves on; it is listed in <program>/TODO.md.
set -euo pipefail
: "${SOLOPEN_AI_API_KEY:?set SOLOPEN_AI_API_KEY}"; : "${GITHUB_TOKEN:?set GITHUB_TOKEN}"
SOLOPEN=${SOLOPEN:-/opt/solopen}; RUNNER=${RUNNER:-$(cd "$(dirname "$0")" && pwd)}
MODEL=${MODEL:-claude-opus-5-5}; BASE_URL=${BASE_URL:-https://api.anthropic.com/v1}
ITERS=${ITERS:-150}; SEGMENTS=${SEGMENTS:-20}
REPO_URL=${REPO_URL:-https://github.com/SolanaOpen/pumpfun-source}
WORK=${WORK:-/work}; REPO=$WORK/pumpfun-source; CAP=$WORK/capture; RUNS=$WORK/runs; LOGS=$WORK/logs
mkdir -p "$WORK" "$CAP" "$RUNS" "$LOGS"
export SOLOPEN_AI_API_KEY
export SOLANA_RPC_URL=${RPC_URL:-https://api.mainnet-beta.solana.com}

askpass=$(mktemp); printf '#!/bin/sh\necho "$GITHUB_TOKEN"\n' > "$askpass"; chmod +x "$askpass"
export GIT_ASKPASS=$askpass GIT_TERMINAL_PROMPT=0
if [ ! -d "$REPO/.git" ]; then git clone -q "$REPO_URL" "$REPO"; else git -C "$REPO" pull -q --rebase; fi
git -C "$REPO" config user.name "solopen-runner"; git -C "$REPO" config user.email "runner@users.noreply.github.com"
git -C "$REPO" remote set-url origin "${REPO_URL/https:\/\//https://x-access-token@}"

programs=${PROGRAMS:-$(python3 -c "import json;print(' '.join(json.load(open('$RUNNER/programs.json'))))")}
cfg() { python3 -c "import json,sys;print(json.load(open('$RUNNER/programs.json'))['$1']['$2'])"; }

run_one() {
  local name=$1 id ws prog pkg tc orig
  id=$(cfg "$name" id); ws=$REPO/$(cfg "$name" workspace); prog=$(cfg "$name" program); pkg=$(cfg "$name" package); tc=$(cfg "$name" toolchain)
  orig=$CAP/$id/program.so
  if [ ! -f "$orig" ]; then (cd "$SOLOPEN" && python3 solopen.py fetch "$id" --capture-root "$CAP" --rpc "$SOLANA_RPC_URL") >> "$LOGS/$name.log" 2>&1; fi
  for seg in $(seq 1 "$SEGMENTS"); do
    local out=$RUNS/$name/seg$(printf %02d "$seg")
    [ -d "$out" ] && continue
    echo "[$name] segment $seg" >> "$LOGS/$name.log"
    ( cd "$SOLOPEN" && python3 solopen.py rebuild --id "$id" --capture-root "$CAP" --workspace "$ws" \
        --program "$prog" --package "$pkg" --toolchain "$tc" --out "$out" --model "$MODEL" --base-url "$BASE_URL" \
        --max-iters "$ITERS" --max-tokens 32000 --target-cache "$WORK/tcache-$name" ) 2>&1 \
      | tee -a "$LOGS/$name.log" \
      | grep --line-buffered -E "^ *accepted:" \
      | while read -r line; do
          sleep 3
          flock "$REPO/.push.lock" python3 "$RUNNER/checkpoint.py" --repo "$REPO" --name "$name" --segment "$out" \
            --label "segment $seg" --original "$orig" --solopen "$SOLOPEN" >> "$LOGS/$name.push.log" 2>&1 || true
        done
    if grep -q "^DONE" "$LOGS/$name.log"; then echo "[$name] byte-identical; stopping" >> "$LOGS/$name.log"; break; fi
  done
}

for p in $programs; do run_one "$p" & done
echo "running: $programs  (logs in $LOGS; Ctrl-C stops everything)"
wait
