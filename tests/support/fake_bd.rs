//! A scripted `bd` for tests: it answers each subcommand with replies the test chooses and
//! records every call, so no test reaches a real Beads database or remote.

use std::fs;

use serde_json::{Value, json};

use super::Fixture;

const BIN: &str = ".fixture-bin";

/// The claimant the example project acts as.
pub const EXAMPLE_CLAIMANT: &str = "example@machine";

/// `bd where` in an initialized workspace.
pub fn available() -> Reply {
    Reply::json(json!({
        "database_path": "/project/.beads/embeddeddolt",
        "path": "/project/.beads",
        "prefix": "wb",
    }))
}

/// `bd sync` that pulled and pushed.
pub fn synced() -> Reply {
    Reply::json(json!({ "status": "ok", "pulled": true, "pushed": true, "attempts": 1 }))
}

/// `bd where` outside any Beads workspace.
pub fn no_workspace() -> Reply {
    Reply::json(json!({
        "error": "no_beads_directory",
        "message": "No active beads workspace found.",
    }))
    .exit(1)
}

/// `bd show` of one item held by `assignee`.
pub fn item(id: &str, status: &str, assignee: &str) -> Reply {
    Reply::json(
        json!([{ "id": id, "title": "Fix the parser", "status": status, "assignee": assignee }]),
    )
}

/// `bd show` of one open item nobody holds.
pub fn open_item(id: &str) -> Reply {
    Reply::json(json!([{ "id": id, "title": "Fix the parser", "status": "open" }]))
}

/// `bd dolt remote list` naming `url` as the `origin` Dolt remote.
pub fn remotes(url: &str) -> Reply {
    Reply::json(json!([{ "name": "origin", "url": url, "sql_url": url, "status": "ok" }]))
}

/// A bare Git repository standing in for the project's GitHub remote, as the Dolt remote URL
/// Beads would report for it. `with_beads_history` gives it a `refs/dolt/data`.
pub fn bare_remote(fixture: &Fixture, with_beads_history: bool) -> String {
    let directory = format!("{BIN}/remote.git");
    fixture.git_command(&["init", "--quiet", "--bare", &directory]);
    if with_beads_history {
        let git = |args: &[&str]| {
            let mut command = vec!["-C", directory.as_str()];
            command.extend_from_slice(args);
            fixture.git_command(&command).trim().to_string()
        };
        let tree = git(&["mktree"]);
        let commit = git(&["commit-tree", &tree, "-m", "dolt data"]);
        git(&["update-ref", "refs/dolt/data", &commit]);
    }
    let root = fixture.root().canonicalize().expect("a fixture root");
    format!("git+file://{}", root.join(directory).display())
}

/// A replica with ready work, an item in progress, memories, and a reachable remote, in which
/// `claimant` can claim and close `wb-1`.
pub fn example_replica(claimant: &str) -> FakeBd {
    FakeBd::new()
        .on("where", available())
        .on("sync", synced())
        .on(
            "ready",
            Reply::json(json!([{ "id": "wb-1", "title": "Fix the parser", "priority": 1, "status": "open" }])),
        )
        .on(
            "list",
            Reply::json(json!([{ "id": "wb-2", "title": "Port status", "status": "in_progress", "assignee": "mark@beta" }])),
        )
        .on(
            "prime",
            Reply::stdout("## Persistent Memories (1)\n\n### fact\nA durable fact.\n"),
        )
        .on("show", item("wb-1", "in_progress", claimant))
        .on("update", item("wb-1", "in_progress", claimant))
        .on("close", Reply::json(json!([{ "id": "wb-1", "status": "closed" }])))
        .on("diff", Reply::stdout("No changes between origin/main and main\n"))
}

/// What one call to the fake `bd` prints and how it exits.
#[derive(Clone, Debug, Default)]
pub struct Reply {
    stdout: String,
    stderr: String,
    exit: i32,
    sleep_seconds: Option<u32>,
}

impl Reply {
    /// Prints `value` as one line of JSON on stdout and exits 0.
    pub fn json(value: Value) -> Reply {
        Reply {
            stdout: format!("{value}\n"),
            ..Reply::default()
        }
    }

    pub fn stdout(text: &str) -> Reply {
        Reply {
            stdout: text.to_string(),
            ..Reply::default()
        }
    }

    pub fn exit(mut self, status: i32) -> Reply {
        self.exit = status;
        self
    }

    pub fn stderr(mut self, text: &str) -> Reply {
        self.stderr = text.to_string();
        self
    }

    /// Waits before answering, as a slow remote would.
    pub fn after_seconds(mut self, seconds: u32) -> Reply {
        self.sleep_seconds = Some(seconds);
        self
    }
}

/// Replies keyed by the subcommand they answer, such as `sync`, `dolt push`, or `update`. The
/// key whose words begin the call's non-flag arguments answers it, the longest key winning. A
/// key with several replies answers successive calls in order, then repeats its last reply.
#[derive(Clone, Debug, Default)]
pub struct FakeBd {
    replies: Vec<(String, Vec<Reply>)>,
}

/// One recorded call: its arguments and the Beads-related environment it ran with.
#[derive(Clone, Debug)]
pub struct BdCall {
    pub args: Vec<String>,
    /// `BEADS_ACTOR`, or `None` when it was unset.
    pub actor: Option<String>,
    /// `DOLT_REMOTE_INFO_BRANCH`, or `None` when it was unset.
    pub info_branch: Option<String>,
    /// `GIT_TERMINAL_PROMPT`, or `None` when it was unset.
    pub terminal_prompt: Option<String>,
}

impl BdCall {
    pub fn line(&self) -> String {
        self.args.join(" ")
    }
}

impl FakeBd {
    pub fn new() -> FakeBd {
        FakeBd::default()
    }

    /// Answers every call to `key` with `reply`, replacing any earlier replies for it.
    pub fn on(self, key: &str, reply: Reply) -> FakeBd {
        self.on_each(key, vec![reply])
    }

    /// Answers successive calls to `key` with `replies`, then repeats the last one.
    pub fn on_each(mut self, key: &str, replies: Vec<Reply>) -> FakeBd {
        assert!(!replies.is_empty(), "{key} needs at least one reply");
        self.replies.retain(|(existing, _)| existing != key);
        self.replies.push((key.to_string(), replies));
        self
    }

    /// Writes the fake as `bd` on the fixture's PATH, replacing any earlier one and its state.
    pub fn install(&self, fixture: &Fixture) {
        let _ = fs::remove_dir_all(fixture.path(&format!("{BIN}/bd.fake")));
        let mut keyed: Vec<(usize, &(String, Vec<Reply>))> =
            self.replies.iter().enumerate().collect();
        keyed.sort_by_key(|(_, (key, _))| std::cmp::Reverse(key.len()));

        let mut patterns = String::new();
        for (index, (key, replies)) in keyed {
            patterns.push_str(&format!(
                "  \" {key} \"*) key=k{index}; last={} ;;\n",
                replies.len() - 1
            ));
            for (position, reply) in replies.iter().enumerate() {
                write_reply(fixture, &format!("k{index}.{position}"), reply);
            }
        }
        fixture.write(&format!("{BIN}/bd.fake/.keep"), "");
        fixture.fake_program("bd", &script(&patterns));
    }
}

fn write_reply(fixture: &Fixture, name: &str, reply: &Reply) {
    let base = format!("{BIN}/bd.fake/{name}");
    fixture
        .write(&format!("{base}.out"), &reply.stdout)
        .write(&format!("{base}.err"), &reply.stderr)
        .write(&format!("{base}.exit"), &reply.exit.to_string());
    if let Some(seconds) = reply.sleep_seconds {
        fixture.write(&format!("{base}.sleep"), &seconds.to_string());
    }
}

fn script(patterns: &str) -> String {
    format!(
        r#"state="$(dirname "$0")/bd.fake"
printf '%s\037%s\037%s' "${{BEADS_ACTOR-<unset>}}" "${{DOLT_REMOTE_INFO_BRANCH-<unset>}}" "${{GIT_TERMINAL_PROMPT-<unset>}}" >> "$state/calls"
for argument in "$@"; do printf '\037%s' "$argument" >> "$state/calls"; done
printf '\036' >> "$state/calls"
words=" "
for argument in "$@"; do
  case "$argument" in -*) ;; *) words="$words$argument " ;; esac
done
case "$words" in
{patterns}  *) echo "fake bd: no reply scripted for: $*" >&2; exit 1 ;;
esac
count=$(cat "$state/$key.count" 2>/dev/null || echo 0)
echo $((count + 1)) > "$state/$key.count"
index=$count
[ "$index" -gt "$last" ] && index=$last
reply="$state/$key.$index"
[ -f "$reply.sleep" ] && sleep "$(cat "$reply.sleep")"
cat "$reply.out"
cat "$reply.err" >&2
exit "$(cat "$reply.exit")""#
    )
}

/// Every call the fake `bd` received, in order.
pub fn bd_calls(fixture: &Fixture) -> Vec<BdCall> {
    let log = fs::read_to_string(fixture.path(&format!("{BIN}/bd.fake/calls"))).unwrap_or_default();
    log.split('\u{1e}')
        .filter(|record| !record.is_empty())
        .map(|record| {
            let mut fields = record.split('\u{1f}').map(str::to_string);
            let recorded = |value: Option<String>| value.filter(|value| value != "<unset>");
            let actor = recorded(fields.next());
            let info_branch = recorded(fields.next());
            let terminal_prompt = recorded(fields.next());
            BdCall {
                args: fields.collect(),
                actor,
                info_branch,
                terminal_prompt,
            }
        })
        .collect()
}

/// The call lines, `sync --json` and the like, without the `dolt` and `where` calls every
/// command makes to check the remote and the workspace.
pub fn bd_steps(fixture: &Fixture) -> Vec<String> {
    bd_calls(fixture)
        .iter()
        .filter(|call| {
            !matches!(
                call.args.first().map(String::as_str),
                Some("dolt" | "where")
            )
        })
        .map(BdCall::line)
        .collect()
}
