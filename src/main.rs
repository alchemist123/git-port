//! Run gitport inside the clone that should RECEIVE commits; the other repo is the "peer".

mod git;

use clap::{Parser, Subcommand};
use console::Style;
use dialoguer::{Confirm, MultiSelect, Select, theme::ColorfulTheme};

#[cfg(windows)]
mod sym {
    pub const CHECK: &str = "[ok]";
    pub const SKIP: &str = "[skip]";
    pub const CROSS: &str = "[x]";
    pub const DASH: &str = "--";
    pub const ARROW_R: &str = "->";
    pub const BOX_H: &str = "-";
    pub const BOX_V: &str = "|";
    pub const BOX_TL: &str = "+";
    pub const BOX_TR: &str = "+";
    pub const BOX_BL: &str = "+";
    pub const BOX_BR: &str = "+";
}

#[cfg(not(windows))]
mod sym {
    pub const CHECK: &str = "\u{2713}";
    pub const SKIP: &str = "\u{2298}";
    pub const CROSS: &str = "\u{2717}";
    pub const DASH: &str = "\u{2014}";
    pub const ARROW_R: &str = "\u{2192}";
    pub const BOX_H: &str = "\u{2500}";
    pub const BOX_V: &str = "\u{2502}";
    pub const BOX_TL: &str = "\u{256d}";
    pub const BOX_TR: &str = "\u{256e}";
    pub const BOX_BL: &str = "\u{2570}";
    pub const BOX_BR: &str = "\u{256f}";
}

#[derive(Parser)]
#[command(
    name = "gitport",
    about = "Port commits between two sibling repos",
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Wire up the peer repo (run once per clone)
    Init {
        peer_url: String,
        #[arg(long, default_value = "peer")]
        name: String,
    },
    /// Show peer commits not yet ported here
    List {
        #[arg(long, short)]
        branch: Option<String>,
    },
    /// Interactively select and cherry-pick peer commits
    Pick {
        #[arg(long, short)]
        branch: Option<String>,
    },
    /// Port a commit (select interactively, or pass a SHA directly)
    Port {
        sha: Option<String>,
        #[arg(long, short)]
        branch: Option<String>,
    },
    /// Push the current branch to origin
    Push,
}

// ── styles ──

fn s_green() -> Style { Style::new().green().bold() }
fn s_yellow() -> Style { Style::new().yellow() }
fn s_cyan() -> Style { Style::new().cyan().bold() }
fn s_dim() -> Style { Style::new().dim() }
fn s_bold() -> Style { Style::new().bold() }
fn s_red() -> Style { Style::new().red().bold() }
fn s_white() -> Style { Style::new().white().bold() }
fn s_magenta() -> Style { Style::new().magenta() }

// ── banner ──

fn print_banner() {
    let h = sym::BOX_H;
    let v = sym::BOX_V;
    let tl = sym::BOX_TL;
    let tr = sym::BOX_TR;
    let bl = sym::BOX_BL;
    let br = sym::BOX_BR;
    let ar = sym::ARROW_R;

    let w = 50;
    let border = h.repeat(w);

    println!();
    println!("  {}{}{}", s_dim().apply_to(tl), s_dim().apply_to(&border), s_dim().apply_to(tr));
    print_box_line(v, &format!("{}", s_cyan().apply_to("gitport")), 7, w);
    print_box_line(v, &format!("{}", s_dim().apply_to("Port commits between two sibling repos")), 38, w);
    println!("  {}{}{}", s_dim().apply_to(bl), s_dim().apply_to(&border), s_dim().apply_to(br));
    println!();
    println!("  {}",
        s_dim().apply_to("Run inside the repo that should RECEIVE commits.")
    );
    println!("  {}",
        s_dim().apply_to("The other repo is the \"peer\" you copy FROM.")
    );
    println!();

    print_cmd("init <url> --name <n>", "Link a peer repo (one-time setup)", "1");
    print_cmd("list", "Show peer commits not yet ported here", "2");
    print_cmd("pick", "Select multiple commits to port", "3");
    print_cmd("port", "Select one commit to port", "4");
    print_cmd("push", "Push current branch to origin", "5");

    println!();
    println!("  {} {}",
        s_bold().apply_to("Typical flow:"),
        s_dim().apply_to(format!(
            "init {ar} list {ar} pick/port {ar} push"
        )),
    );
    println!();
    println!("  {}  gitport init https://github.com/org/peer.git --name peer",
        s_dim().apply_to("$"),
    );
    println!("  {}  gitport list",
        s_dim().apply_to("$"),
    );
    println!("  {}  gitport pick",
        s_dim().apply_to("$"),
    );
    println!("  {}  gitport push",
        s_dim().apply_to("$"),
    );
    println!();
    println!("  {} gitport <command> --help",
        s_dim().apply_to("Run"),
    );
    println!();
}

fn print_box_line(v: &str, content: &str, content_len: usize, width: usize) {
    let padding = width - 2 - content_len;
    println!("  {} {}{}{}", s_dim().apply_to(v), content, " ".repeat(padding), s_dim().apply_to(v));
}

fn print_cmd(cmd: &str, desc: &str, num: &str) {
    println!("  {}  {}  {}",
        s_magenta().apply_to(format!("{num}.")),
        s_white().apply_to(format!("{:<28}", format!("gitport {cmd}"))),
        s_dim().apply_to(desc),
    );
}

// ── helpers ──

fn check_git_installed() -> anyhow::Result<()> {
    git::git(&["--version"]).map_err(|_| {
        anyhow::anyhow!(
            "git is not installed or not in PATH.\n\
             Install it from https://git-scm.com/downloads"
        )
    })?;
    Ok(())
}

fn peer() -> anyhow::Result<String> {
    git::git(&["config", "gitport.peer"])
        .map_err(|_| anyhow::anyhow!("no peer configured -- run `gitport init` first"))
}

fn current_branch() -> anyhow::Result<String> {
    git::git(&["rev-parse", "--abbrev-ref", "HEAD"])
}

fn peer_branches() -> anyhow::Result<Vec<String>> {
    let p = peer()?;
    let output = git::git(&["branch", "-r", "--list", &format!("{p}/*")])?;
    let branches: Vec<String> = output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.contains("->"))
        .collect();
    if branches.is_empty() {
        anyhow::bail!("no branches found for peer '{p}' -- try `gitport init` again");
    }
    Ok(branches)
}

fn pick_peer_branch(branches: &[String]) -> anyhow::Result<String> {
    if branches.len() == 1 {
        return Ok(branches[0].clone());
    }
    let theme = ColorfulTheme::default();
    let idx = Select::with_theme(&theme)
        .with_prompt("Which peer branch to port from?")
        .items(branches)
        .default(0)
        .interact()?;
    Ok(branches[idx].clone())
}

fn resolve_peer_branch(explicit: Option<String>) -> anyhow::Result<String> {
    if let Some(b) = explicit {
        let p = peer()?;
        return Ok(format!("{p}/{b}"));
    }
    let branches = peer_branches()?;
    pick_peer_branch(&branches)
}

fn outstanding(remote_branch: &str) -> anyhow::Result<Vec<(String, String)>> {
    let p = peer()?;
    let b = current_branch()?;

    if !git::git_live(&["fetch", &p])? {
        anyhow::bail!("git fetch {p} failed");
    }

    let output = git::git(&["cherry", "-v", &b, remote_branch])?;

    let commits = output
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("+ ")?;
            let (sha, subject) = rest.split_once(' ')?;
            Some((sha.to_string(), subject.to_string()))
        })
        .collect();

    Ok(commits)
}

fn ensure_clean() -> anyhow::Result<()> {
    let status = git::git(&["status", "--porcelain"])?;
    if !status.is_empty() {
        anyhow::bail!("working tree not clean -- commit or stash changes first");
    }
    Ok(())
}

fn cherry_pick_one(sha: &str) -> anyhow::Result<()> {
    let short = &sha[..7.min(sha.len())];

    println!("  {} cherry-pick {short}", s_dim().apply_to("->"));

    if git::git_live(&["cherry-pick", "-x", sha])? {
        println!("  {} {short}", s_green().apply_to(sym::CHECK));
        return Ok(());
    }

    let output = git::git_output(&["status", "--porcelain"])?;
    let status_text = String::from_utf8_lossy(&output.stdout);
    if status_text.trim().is_empty() {
        println!("  {} {short} {} already applied, skipping", s_yellow().apply_to(sym::SKIP), sym::DASH);
        git::git(&["cherry-pick", "--skip"])?;
        return Ok(());
    }

    anyhow::bail!(
        "conflict on {short}. Resolve it, then run:\n  \
         git cherry-pick --continue   (to keep)\n  \
         git cherry-pick --abort      (to undo)\n\
         Then re-run gitport."
    );
}

fn print_commit_list(commits: &[(String, String)]) {
    for (i, (sha, subject)) in commits.iter().enumerate() {
        println!(
            "  {} {}  {}",
            s_dim().apply_to(format!("{:>3}.", i + 1)),
            s_yellow().apply_to(&sha[..7]),
            subject,
        );
    }
}

fn confirm_and_apply(mut selected: Vec<(String, String)>) -> anyhow::Result<()> {
    let theme = ColorfulTheme::default();

    loop {
        println!();
        println!("{}", s_bold().apply_to(format!(
            "Will port {} commit(s):",
            selected.len()
        )));
        println!();
        print_commit_list(&selected);
        println!();

        if selected.len() > 1 {
            let choices = &["Yes, port these commits", "Remove some commits", "Cancel"];
            let choice = Select::with_theme(&theme)
                .with_prompt("Proceed?")
                .items(choices)
                .default(0)
                .interact()?;

            match choice {
                0 => break,
                1 => {
                    let labels: Vec<String> = selected
                        .iter()
                        .map(|(sha, subject)| format!("{} {}", &sha[..7], subject))
                        .collect();

                    let to_remove = MultiSelect::with_theme(&theme)
                        .with_prompt("Select commits to REMOVE (Space toggle, Enter confirm)")
                        .items(&labels)
                        .interact()?;

                    if to_remove.is_empty() {
                        println!("  {} nothing removed", s_dim().apply_to(sym::DASH));
                        continue;
                    }

                    for idx in to_remove.iter().rev() {
                        let (sha, subject) = &selected[*idx];
                        println!("  {} {} {}", s_red().apply_to(sym::CROSS), &sha[..7], subject);
                        selected.remove(*idx);
                    }

                    if selected.is_empty() {
                        println!("All commits removed. Nothing to port.");
                        return Ok(());
                    }

                    continue;
                }
                _ => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
        } else {
            if !Confirm::with_theme(&theme)
                .with_prompt("Port this commit?")
                .default(true)
                .interact()?
            {
                println!("Cancelled.");
                return Ok(());
            }
            break;
        }
    }

    println!();
    println!("{}", s_bold().apply_to(format!("Porting {} commit(s)...", selected.len())));
    println!();

    for (sha, _) in &selected {
        cherry_pick_one(sha)?;
    }

    println!();
    println!(
        "{} {} commit(s) ported. Run `gitport push` when ready.",
        s_green().apply_to(sym::CHECK),
        selected.len()
    );
    Ok(())
}

// ── commands ──

fn init(peer_url: &str, name: &str) -> anyhow::Result<()> {
    git::git(&["rev-parse", "--git-dir"])
        .map_err(|_| anyhow::anyhow!("not inside a git repository"))?;

    if git::git(&["remote", "get-url", name]).is_ok() {
        git::git(&["remote", "set-url", name, peer_url])?;
    } else {
        git::git(&["remote", "add", name, peer_url])?;
    }

    git::git(&["config", "gitport.peer", name])?;
    git::git(&["config", "rerere.enabled", "true"])?;

    if !git::git_live(&["fetch", name])? {
        anyhow::bail!("git fetch {name} failed");
    }

    println!("{} Linked peer '{}' -> {}", s_green().apply_to(sym::CHECK), name, peer_url);
    Ok(())
}

fn list(branch_flag: Option<String>) -> anyhow::Result<()> {
    let remote_branch = resolve_peer_branch(branch_flag)?;
    let commits = outstanding(&remote_branch)?;

    if commits.is_empty() {
        println!("{}", s_green().apply_to("Nothing to port -- peer branch is fully synced here."));
        return Ok(());
    }

    println!();
    println!("{}", s_bold().apply_to(format!("Commits in {remote_branch} not yet here:")));
    println!();
    print_commit_list(&commits);
    println!();
    println!("{} commit(s) available", commits.len());
    Ok(())
}

fn pick(branch_flag: Option<String>) -> anyhow::Result<()> {
    ensure_clean()?;

    let remote_branch = resolve_peer_branch(branch_flag)?;
    let mut commits = outstanding(&remote_branch)?;

    if commits.is_empty() {
        println!("{}", s_green().apply_to("Nothing to port."));
        return Ok(());
    }

    commits.reverse();

    let labels: Vec<String> = commits
        .iter()
        .map(|(sha, subject)| format!("{} {}", &sha[..7], subject))
        .collect();

    let theme = ColorfulTheme::default();
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select commits to port (arrow keys move, Space toggle, Enter confirm)")
        .items(&labels)
        .interact()?;

    if selections.is_empty() {
        println!("Nothing selected.");
        return Ok(());
    }

    let selected: Vec<(String, String)> = selections
        .into_iter()
        .map(|i| commits[i].clone())
        .collect();

    confirm_and_apply(selected)
}

fn port(sha: Option<String>, branch_flag: Option<String>) -> anyhow::Result<()> {
    ensure_clean()?;

    if let Some(sha) = sha {
        let selected = vec![(sha.clone(), commit_subject(&sha)?)];
        return confirm_and_apply(selected);
    }

    let remote_branch = resolve_peer_branch(branch_flag)?;
    let mut commits = outstanding(&remote_branch)?;

    if commits.is_empty() {
        println!("{}", s_green().apply_to("Nothing to port."));
        return Ok(());
    }

    commits.reverse();

    let labels: Vec<String> = commits
        .iter()
        .map(|(sha, subject)| format!("{} {}", &sha[..7], subject))
        .collect();

    let theme = ColorfulTheme::default();
    let idx = Select::with_theme(&theme)
        .with_prompt("Select a commit to port (arrow keys move, Enter select)")
        .items(&labels)
        .default(0)
        .interact()?;

    let selected = vec![commits[idx].clone()];
    confirm_and_apply(selected)
}

fn commit_subject(sha: &str) -> anyhow::Result<String> {
    git::git(&["log", "-1", "--format=%s", sha])
}

fn push() -> anyhow::Result<()> {
    let branch = current_branch()?;
    if !git::git_live(&["push", "origin", &branch])? {
        anyhow::bail!("push to origin/{branch} failed");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    #[cfg(windows)]
    let _ = console::Term::stdout();

    let cli = Cli::parse();

    match cli.cmd {
        None => {
            print_banner();
            Ok(())
        }
        Some(cmd) => {
            check_git_installed()?;
            match cmd {
                Cmd::Init { peer_url, name } => init(&peer_url, &name),
                Cmd::List { branch } => list(branch),
                Cmd::Pick { branch } => pick(branch),
                Cmd::Port { sha, branch } => port(sha, branch),
                Cmd::Push => push(),
            }
        }
    }
}
