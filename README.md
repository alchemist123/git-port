# gitport

Port commits from one git repo into another via cherry-pick.

Works on **Windows**, **macOS**, and **Linux**.

**Key idea:** You always run gitport inside the repo that should *receive* commits.
The other repo (the one you're copying commits *from*) is called the "peer."

gitport works with any git URL -- local paths, GitHub HTTPS, GitHub SSH, GitLab, etc.

```
repo-a  (has new commits)  ---- peer ---->  repo-b  (you run gitport here)
repo-b  (has new commits)  ---- peer ---->  repo-a  (you run gitport here)
```

You can port in **both directions**. Just run gitport from whichever repo should receive the commits.

## Prerequisites

- **Git 2.x+** -- must be in your PATH
  - macOS: `brew install git` or Xcode Command Line Tools
  - Linux: `sudo apt install git` (Debian/Ubuntu) or `sudo dnf install git` (Fedora)
  - Windows: [Git for Windows](https://gitforwindows.org/) (adds `git` to PATH)
- **Rust toolchain** (cargo, rustc) -- [Install from rustup.rs](https://rustup.rs/)
  - Works on all three platforms. On Windows, the installer prompts you to install the Visual C++ build tools if needed.

## Build

### macOS / Linux

```bash
cd gitport
cargo build --release

# Optional: copy to PATH so you can type "gitport" anywhere
sudo cp target/release/gitport /usr/local/bin/
```

### Windows (PowerShell or Command Prompt)

```powershell
cd gitport
cargo build --release

# Optional: copy to a folder in your PATH
copy target\release\gitport.exe C:\Users\%USERNAME%\.cargo\bin\
```

> **Tip:** `cargo install --path .` also works on all platforms -- it builds
> and copies the binary to `~/.cargo/bin/` automatically.

If you don't copy to PATH, use the full path to the binary:

| OS | Full path |
|---|---|
| macOS / Linux | `./target/release/gitport` |
| Windows | `.\target\release\gitport.exe` |

## Commands at a glance

| Command | What it does |
|---|---|
| `gitport init <peer-url> --name <name>` | Connect to the peer repo (run once) |
| `gitport list` | Show peer commits you don't have yet |
| `gitport list -b main` | Same, but for a specific peer branch |
| `gitport pick` | Checkbox UI -- select multiple commits to bring in |
| `gitport pick -b main` | Same, but for a specific peer branch |
| `gitport port` | Select ONE commit interactively, confirm, then port it |
| `gitport port <sha>` | Port a specific commit by SHA (still shows confirmation) |
| `gitport push` | Push your branch to origin |

> **Branch selection:** If the peer has multiple branches and you don't pass
> `-b` / `--branch`, gitport shows an interactive menu to pick one.
> If the peer only has one branch, it's selected automatically.

---

## What can you use as a peer URL?

gitport accepts anything `git remote add` accepts:

| Type | Example |
|---|---|
| Local path (macOS/Linux) | `/Users/you/projects/other-repo` |
| Local path (Windows) | `C:\Users\you\projects\other-repo` |
| GitHub HTTPS | `https://github.com/org/repo.git` |
| GitHub SSH | `git@github.com:org/repo.git` |
| GitLab / other | `https://gitlab.com/org/repo.git` |

---

## Example 1: Local repos (A to B)

Port commits from a local repo-a into repo-b.

### macOS / Linux

```bash
# >>> Run in: repo-b <<<
cd /path/to/repo-b

gitport init /path/to/repo-a --name repo-a
gitport list
gitport port
gitport push
```

### Windows

```powershell
# >>> Run in: repo-b <<<
cd C:\Users\you\projects\repo-b

gitport init C:\Users\you\projects\repo-a --name repo-a
gitport list
gitport port
gitport push
```

---

## Example 2: Local repos (B to A) -- the other direction

Same tool, just run it from repo-a instead.

### macOS / Linux

```bash
# >>> Run in: repo-a <<<
cd /path/to/repo-a

gitport init /path/to/repo-b --name repo-b
gitport list
gitport port
gitport push
```

### Windows

```powershell
# >>> Run in: repo-a <<<
cd C:\Users\you\projects\repo-a

gitport init C:\Users\you\projects\repo-b --name repo-b
gitport list
gitport port
gitport push
```

> **Each repo has its own peer.** repo-b's peer is repo-a, and repo-a's peer is repo-b.
> The `init` in each repo is independent -- they don't interfere with each other.

---

## Example 3: GitHub remote repos (A to B)

Port commits from a GitHub repo into your local clone of another GitHub repo.
This works the same on all platforms.

```bash
# >>> Run in: your local clone of repo-b <<<
cd /path/to/your/repo-b

# Using HTTPS:
gitport init https://github.com/org/repo-a.git --name repo-a

# Or using SSH:
gitport init git@github.com:org/repo-a.git --name repo-a

gitport list              # fetches from GitHub, shows what's missing
gitport port              # select and port one commit
gitport pick              # or select and port multiple commits
gitport push              # push repo-b to GitHub
```

> **Note:** gitport runs `git fetch` every time you run `list`, `pick`, or `port`,
> so it always sees the latest commits from the peer -- even if someone just pushed
> to the GitHub repo.

---

## Example 4: GitHub remote repos (B to A) -- the other direction

```bash
# >>> Run in: your local clone of repo-a <<<
cd /path/to/your/repo-a

gitport init https://github.com/org/repo-b.git --name repo-b
gitport list
gitport port
gitport push
```

---

## Detailed walkthrough of the interactive flow

### `gitport port` (single commit)

**Step 1 -- Select:** A list of available commits appears. Use arrow keys and Enter:

```
? Select a commit to port (arrow keys move, Enter select)
> abc1234 feat: add hello
  def5678 feat: add world
```

**Step 2 -- Confirm:** gitport shows what you picked and asks for confirmation:

```
Will port 1 commit(s):

    1. abc1234  feat: add hello

? Port this commit? (y/n) > yes
```

**Step 3 -- Done:**

```
Porting 1 commit(s)...

  -> cherry-pick abc1234
  [ok] abc1234

[ok] 1 commit(s) ported. Run `gitport push` when ready.
```

### `gitport pick` (multiple commits)

**Step 1 -- Select:** A checkbox list appears. Toggle with Space, confirm with Enter:

```
? Select commits to port (arrow keys move, Space toggle, Enter confirm)
> [x] abc1234 feat: add hello
  [x] def5678 feat: add world
```

**Step 2 -- Confirm:** gitport shows your selection. You can review, remove some, or cancel:

```
Will port 2 commit(s):

    1. abc1234  feat: add hello
    2. def5678  feat: add world

? Proceed?
> Yes, port these commits
  Remove some commits
  Cancel
```

If you choose **"Remove some commits"**, another checkbox appears to deselect any you don't want. Then you're back to the confirmation screen.

**Step 3 -- Done:**

```
Porting 2 commit(s)...

  -> cherry-pick abc1234
  [ok] abc1234
  -> cherry-pick def5678
  [ok] def5678

[ok] 2 commit(s) ported. Run `gitport push` when ready.
```

---

## What happens in edge cases

### Already-applied commits

If you try to port a commit whose changes already exist (same patch content, different SHA), gitport auto-skips it:

```
  -> cherry-pick 926fabd
  [skip] 926fabd -- already applied, skipping
```

No error, no manual intervention needed.

### Dirty working tree

If you have uncommitted changes, `pick` and `port` refuse to run:

```
Error: working tree not clean -- commit or stash changes first
```

Fix it with `git stash` or `git commit` first.

### Cherry-pick conflict

If a commit conflicts with your local changes, gitport stops and tells you:

```
conflict on abc1234. Resolve it, then run:
  git cherry-pick --continue   (to keep)
  git cherry-pick --abort      (to undo)
Then re-run gitport.
```

To resolve:
1. Open the conflicting file(s) and fix the `<<<<<<<` markers
2. `git add <fixed-file>`
3. `git cherry-pick --continue`
4. Re-run `gitport pick` or `gitport port` for any remaining commits

### Re-running list is always safe

`list` compares by patch content (patch-id), not by SHA. A commit that was
cherry-picked -- even with a different SHA and timestamp -- won't show up again.

---

## Quick-reference cheat sheet

```bash
# --- BUILD (once, any platform) ---
cd gitport
cargo build --release
# or: cargo install --path .

# --- LOCAL: A to B ---
cd /path/to/repo-b                                          # or C:\...\repo-b on Windows
gitport init /path/to/repo-a --name repo-a                  # or C:\...\repo-a on Windows
gitport list && gitport port && gitport push

# --- LOCAL: B to A ---
cd /path/to/repo-a
gitport init /path/to/repo-b --name repo-b
gitport list && gitport port && gitport push

# --- GITHUB: A to B ---
cd /path/to/repo-b
gitport init https://github.com/org/repo-a.git --name repo-a
gitport list && gitport port && gitport push

# --- GITHUB: B to A ---
cd /path/to/repo-a
gitport init https://github.com/org/repo-b.git --name repo-b
gitport list && gitport port && gitport push
```

## Install from GitHub Releases (no Rust needed)

Once the CI/CD is set up (see below), every tagged release produces pre-built
binaries. Users can download and run without installing Rust.

### macOS (Apple Silicon)

```bash
curl -L https://github.com/YOUR_ORG/gitport/releases/latest/download/gitport-macos-arm64 -o gitport
chmod +x gitport
sudo mv gitport /usr/local/bin/
```

### macOS (Intel)

```bash
curl -L https://github.com/YOUR_ORG/gitport/releases/latest/download/gitport-macos-amd64 -o gitport
chmod +x gitport
sudo mv gitport /usr/local/bin/
```

### Linux

```bash
curl -L https://github.com/YOUR_ORG/gitport/releases/latest/download/gitport-linux-amd64 -o gitport
chmod +x gitport
sudo mv gitport /usr/local/bin/
```

### Windows (PowerShell)

```powershell
Invoke-WebRequest -Uri https://github.com/YOUR_ORG/gitport/releases/latest/download/gitport-windows-amd64.exe -OutFile gitport.exe
Move-Item gitport.exe C:\Users\$env:USERNAME\.cargo\bin\
```

> Replace `YOUR_ORG/gitport` with your actual GitHub repo path.

---

## CI/CD: GitHub Actions setup (step-by-step)

This repo includes two workflow files that handle CI and releases automatically:

```
.github/
  workflows/
    ci.yml        <-- runs on every push/PR: clippy + test + build (all 3 platforms)
    release.yml   <-- runs when you push a version tag: builds + creates GitHub Release
```

### What the release produces

| File | Platform |
|---|---|
| `gitport-linux-amd64` | Linux x86_64 |
| `gitport-macos-arm64` | macOS Apple Silicon (M1/M2/M3/M4) |
| `gitport-macos-amd64` | macOS Intel |
| `gitport-windows-amd64.exe` | Windows x86_64 |

### Step-by-step: push your first release

**Step 1: Create a GitHub repo**

Go to https://github.com/new and create a new repo (e.g., `gitport`). Leave it empty -- don't add a README or .gitignore from GitHub.

**Step 2: Connect your local repo to GitHub**

```bash
cd /Users/amalvs/Documents/projects/repo-manage/gitport

# Point origin to your new GitHub repo
git remote add origin https://github.com/YOUR_USERNAME/gitport.git
```

Or if you already have an origin:

```bash
git remote set-url origin https://github.com/YOUR_USERNAME/gitport.git
```

**Step 3: Commit everything and push**

```bash
# Stage all files
git add .gitignore Cargo.toml Cargo.lock src/ README.md .github/

# Create the first commit
git commit -m "initial release"

# Push to GitHub (use main or master, whichever your branch is)
git branch -M main
git push -u origin main
```

At this point the **CI workflow** runs automatically. Go to your repo on GitHub,
click the **Actions** tab, and you should see the "CI" workflow running. It will
build and check on all 3 platforms.

**Step 4: Create a version tag to trigger the release**

```bash
# Tag the current commit as v0.1.0
git tag v0.1.0

# Push the tag to GitHub
git push origin v0.1.0
```

This triggers the **Release workflow**. It will:
1. Build the binary on Linux, macOS (ARM + Intel), and Windows
2. Create a GitHub Release page at `https://github.com/YOUR_USERNAME/gitport/releases`
3. Attach all 4 binaries to the release

**Step 5: Verify**

1. Go to your repo on GitHub
2. Click **Actions** -- you should see the "Release" workflow running
3. Wait for it to finish (usually 3-5 minutes)
4. Click **Releases** on the right sidebar (or go to `/releases`)
5. You should see `v0.1.0` with 4 binary files attached

### Future releases

Every time you want to release a new version:

```bash
# Make your changes, commit them
git add -A
git commit -m "feat: whatever you changed"
git push

# Tag and push to trigger a release
git tag v0.2.0
git push origin v0.2.0
```

### Workflow permissions

The release workflow needs permission to create releases. This is handled by the
`permissions: contents: write` line in `release.yml`. If your repo uses restricted
permissions, go to:

> Settings > Actions > General > Workflow permissions > **Read and write permissions**

---

## Platform notes

| | macOS | Linux | Windows |
|---|---|---|---|
| Terminal | Terminal.app, iTerm2 | Any terminal | Windows Terminal, PowerShell, cmd.exe |
| Colors | Yes | Yes | Yes (ANSI enabled automatically) |
| Symbols | Unicode (checkmarks) | Unicode (checkmarks) | ASCII fallback ([ok], [skip], [x]) |
| Git | `brew install git` or Xcode CLI | `apt install git` | [gitforwindows.org](https://gitforwindows.org/) |
| Rust | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Same | [rustup.rs](https://rustup.rs/) installer |

## How it works under the hood

- **`git cherry -v`** compares commits by *patch content* (patch-id), not by SHA. Two commits with the same code change -- even different SHAs -- are treated as equivalent. This is how gitport knows what's already ported.

- **`git cherry-pick -x`** copies a commit and appends `(cherry picked from commit ...)` to the message, leaving a cross-repo paper trail.

- **`git config rerere.enabled true`** (set by `init`) tells git to remember conflict resolutions. If the same conflict comes up again, git resolves it automatically.
