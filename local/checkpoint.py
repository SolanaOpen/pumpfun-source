#!/usr/bin/env python3
"""Push a loop's latest kept state to the repo, safely while the loop runs.

The loop writes an edit into the workspace before scoring it, so a plain copy can
catch an unscored edit. This snapshots the workspace and commits only if every
file matches the source manifest of the last attempt the loop kept.

  python3 checkpoint.py --repo <checkout> --name mayhem --segment <segment dir> --label "segment 4"
"""
import argparse, hashlib, json, re, shutil, subprocess, sys, tempfile
from pathlib import Path

DOCS = {"README.md", "TODO.md"}


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True, type=Path)
    ap.add_argument("--name", required=True)
    ap.add_argument("--segment", required=True, type=Path)
    ap.add_argument("--label", required=True)
    ap.add_argument("--original", required=True, type=Path)
    ap.add_argument("--solopen", required=True, type=Path)
    ap.add_argument("--no-push", action="store_true")
    a = ap.parse_args()
    cfg = json.loads((Path(__file__).parent / "programs.json").read_text())[a.name]
    ws = a.repo / cfg["workspace"]
    hist_path = a.segment / "history.json"
    if not hist_path.exists():
        print("no history yet"); return 0
    hist = json.loads(hist_path.read_text())
    kept = [h for h in hist if str(h.get("outcome", "")).startswith("accepted")]
    if not kept:
        print("nothing kept yet in this segment"); return 0
    last = kept[-1]
    run = a.segment / f"attempt-{last['attempt']:03d}"
    trial = json.loads((run / "trial-result.json").read_text())
    manifest = json.loads((run / "source-manifest.json").read_text())["files"]

    snap = Path(tempfile.mkdtemp(prefix=f"{a.name}-snap-")) / "ws"
    shutil.copytree(ws, snap, ignore=shutil.ignore_patterns("target", ".git"))
    # Verify the build inputs (source, manifests, lockfile, IDLs). README/TODO/targets are
    # outputs this script regenerates, so they are not part of the check.
    is_input = lambda rel: not (rel in DOCS or rel.startswith("targets/"))
    present = {str(p.relative_to(snap)) for p in snap.rglob("*") if p.is_file() and is_input(str(p.relative_to(snap)))}
    expected = {f["path"] for f in manifest if is_input(f["path"])}
    bad = [f["path"] for f in manifest if is_input(f["path"])
           and (not (snap / f["path"]).is_file() or sha(snap / f["path"]) != f["sha256"])]
    if bad or present != expected:
        print(f"workspace is mid-edit (differs from attempt {last['attempt']}); not pushing"); return 1

    shutil.rmtree(ws / "targets", ignore_errors=True)
    shutil.copytree(run / "targets", ws / "targets")
    leads = ws / "LEADS.md"
    todo = [sys.executable, str(a.repo / "tools/todo.py"), "--targets", str(run / "targets/targets.json"),
            "--original", str(a.original), "--solopen", str(a.solopen), "--out", str(ws / "TODO.md")]
    if leads.exists():
        todo += ["--leads", str(leads)]
    subprocess.run(todo, check=True)

    pct = trial["coverage_pct"]
    calls = sum(1 for h in hist if h.get("attempt"))
    row = f"| AI rebuild loop, {a.label}: {calls} model calls, {len(kept)} edits kept so far | {pct:.2f}% |"
    readme = ws / "README.md"
    text = readme.read_text() if readme.exists() else f"# {a.name}\n\n## Progress\n\n| Step | Coverage |\n|---|---:|\n"
    if f"| AI rebuild loop, {a.label}:" in text:
        text = re.sub(rf"^\| AI rebuild loop, {re.escape(a.label)}:.*$", row, text, flags=re.M)
    else:
        lines = text.rstrip("\n").split("\n")
        rows = [i for i, l in enumerate(lines) if re.match(r"^\| .* \| [0-9.]+% \|$", l) or l.startswith("|---|")]
        lines.insert((rows[-1] + 1) if rows else len(lines), row)
        text = "\n".join(lines) + "\n"
    readme.write_text(text)

    git = lambda *x: subprocess.run(["git", "-C", str(a.repo), *x], check=True, capture_output=True, text=True)
    git("add", "-A", str(ws))
    if not git("status", "--porcelain", str(ws)).stdout.strip():
        print("no changes"); return 0
    leak = subprocess.run(["git", "-C", str(a.repo), "grep", "-n", "-E", "--cached", "-e", "sk-ant-[A-Za-z0-9_-]{20,}", "-e", "(ghp|github_pat)_[A-Za-z0-9_]{20,}", "--", str(ws)],
                          capture_output=True, text=True).stdout
    if leak.strip():
        print("refusing to commit: a key is in the tree:\n" + leak[:300]); return 2
    msg = (f"{a.name}: AI loop {a.label}, {pct:.2f}% of the original's code\n\n"
           f"Workspace after model call {last['attempt']} ({len(kept)} edits kept so far), verified against that "
           "build's source manifest before committing.\n")
    subprocess.run(["git", "-C", str(a.repo), "commit", "-q", "-m", msg], check=True)
    if not a.no_push:
        subprocess.run(["git", "-C", str(a.repo), "push", "-q", "origin", "HEAD"], check=True)
    print(f"{'committed' if a.no_push else 'pushed'} {a.name} {pct:.2f}%")
    return 0


if __name__ == "__main__":
    sys.exit(main())
