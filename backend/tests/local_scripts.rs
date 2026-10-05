#![cfg(target_os = "linux")]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("shoken-local-{}", uuid::Uuid::new_v4()));
        for directory in ["scripts", "backend", "frontend", "bin"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for file in [
            "scripts/start-local.sh",
            "scripts/stop-local.sh",
            "frontend/Trunk.toml",
        ] {
            fs::copy(source.join(file), root.join(file)).unwrap();
        }
        fs::write(
            root.join("shell-env"),
            r#"
kill() {
  if [[ "$1" == "-0" ]]; then
    [[ -f "${MOCK_DIR}/alive-$2" ]]
    return
  fi
  [[ "$1" == "-9" ]] && shift
  for pid in "$@"; do
    printf '%s\n' "$pid" >> "${MOCK_DIR}/killed"
    rm -f "${MOCK_DIR}/alive-${pid}"
  done
}
sleep() { :; }
command() {
  if [[ "$1" == "-v" && "$2" =~ ^(lsof|ss|fuser)$ ]]; then
    [[ " ${SCANNERS:-lsof} " == *" $2 "* ]]
  else
    builtin command "$@"
  fi
}
"#,
        )
        .unwrap();
        let sandbox = Self { root };
        sandbox.tool("ps", r#"
if [[ "$*" == *"lstart="* ]]; then
  printf '%s\n' 'Mon Oct  5 12:00:00 2026'
elif [[ "$*" == *"comm="* ]]; then
  if [[ " $* " == *" 42 "* ]]; then printf '%s\n' "${FRONT_NAME:-trunk}"; else printf '%s\n' "${BACK_NAME:-backend}"; fi
else
  if [[ " $* " == *" 42 "* ]]; then printf '%s\n' "${FRONT_COMMAND:-trunk serve --port 8081}"; else printf '%s\n' "${BACK_COMMAND:-target/debug/backend}"; fi
fi
"#);
        sandbox.tool(
            "readlink",
            r#"
case "$*" in
  */42/cwd) printf '%s\n' "${FRONT_CWD:-${MOCK_DIR}/frontend}" ;;
  */43/cwd) printf '%s\n' "${BACK_CWD:-${MOCK_DIR}/backend}" ;;
  *) exit 1 ;;
esac
"#,
        );
        sandbox.tool("lsof", r#"
printf '%s\n' "lsof $*" >> "${MOCK_DIR}/scan"
if [[ "$*" == *":8081"* ]]; then printf '%s\n' "${FRONT_SCAN_PIDS:-}"; else printf '%s\n' "${BACK_SCAN_PIDS:-}"; fi
exit "${LSOF_STATUS:-0}"
"#);
        sandbox.tool(
            "ss",
            r#"
printf '%s\n' "ss $*" >> "${MOCK_DIR}/scan"
if [[ "$*" == *":8081"* ]]; then pid="${FRONT_SCAN_PIDS:-}"; else pid="${BACK_SCAN_PIDS:-}"; fi
[[ -n "$pid" ]] && printf 'users:(("server",pid=%s,fd=3))\n' "$pid"
exit "${SS_STATUS:-0}"
"#,
        );
        sandbox.tool("fuser", r#"
printf '%s\n' "fuser $*" >> "${MOCK_DIR}/scan"
if [[ "$*" == *"8081"* ]]; then printf '%s\n' "${FRONT_SCAN_PIDS:-}"; else printf '%s\n' "${BACK_SCAN_PIDS:-}"; fi
exit "${FUSER_STATUS:-0}"
"#);
        sandbox.tool(
            "curl",
            r#"
printf '%s\n' "$*" >> "${MOCK_DIR}/curl-calls"
exit "${CURL_STATUS:-22}"
"#,
        );
        sandbox.tool("docker", "exit 0\n");
        sandbox.tool("make", "exit 0\n");
        sandbox.tool(
            "cargo",
            r#"
printf 'args=%s PORT=%s BACKEND_URL=%s\n' "$*" "${PORT:-}" "${BACKEND_URL:-}" >> "${MOCK_DIR}/cargo-env"
for _ in $(seq 1 1000000); do
  [[ -f "${MOCK_DIR}/trunk-done" ]] && break
done
"#,
        );
        sandbox.tool(
            "trunk",
            r#"
printf '%s\n' "$*" >> "${MOCK_DIR}/trunk-args"
config=""
prev=""
for arg in "$@"; do
  [[ "$prev" == "--config" ]] && config="$arg"
  prev="$arg"
done
[[ -n "$config" ]] && cp "$config" "${MOCK_DIR}/trunk-config"
for f in /tmp/shoken-backend-dev-*; do
  [[ -e "$f" ]] && basename "$f"
done >> "${MOCK_DIR}/backend-pid-names"
touch "${MOCK_DIR}/trunk-done"
"#,
        );
        sandbox
    }

    fn tool(&self, name: &str, body: &str) {
        let path = self.root.join("bin").join(name);
        fs::write(&path, format!("#!/usr/bin/env bash\n{body}")).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn ledger(&self, name: &str, pid: u32) {
        fs::write(
            self.root.join(name),
            format!("{pid}\nMon Oct  5 12:00:00 2026\n"),
        )
        .unwrap();
        fs::write(self.root.join(format!("alive-{pid}")), "").unwrap();
    }

    fn run(&self, script: &str, env: &[(&str, &str)]) -> Output {
        let mut command = Command::new("bash");
        command
            .arg(self.root.join("scripts").join(script))
            .env_clear()
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    self.root.join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env("HOME", &self.root)
            .env("LANG", "C")
            .env("BASH_ENV", self.root.join("shell-env"))
            .env("MOCK_DIR", &self.root);
        if script == "stop-local.sh" {
            command
                .arg("--keep-db")
                .env("BACKEND_PID_FILE", self.root.join("backend.pid"))
                .env("FRONTEND_PID_FILE", self.root.join("frontend.pid"));
        }
        for (name, value) in env {
            command.env(name, value);
        }
        command.output().unwrap()
    }

    fn text(&self, name: &str) -> String {
        fs::read_to_string(self.root.join(name)).unwrap_or_default()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn scanner_no_matches_is_success_without_a_warning() {
    for (scanner, status) in [
        ("lsof", "LSOF_STATUS"),
        ("ss", "SS_STATUS"),
        ("fuser", "FUSER_STATUS"),
    ] {
        let sandbox = Sandbox::new();
        let output = sandbox.run("stop-local.sh", &[("SCANNERS", scanner), (status, "1")]);
        assert_success(&output);
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(sandbox.text("killed").is_empty());
    }
}

#[test]
fn scanner_failure_is_not_reported_as_no_matches() {
    for (scanner, status) in [
        ("lsof", "LSOF_STATUS"),
        ("ss", "SS_STATUS"),
        ("fuser", "FUSER_STATUS"),
    ] {
        let sandbox = Sandbox::new();
        let output = sandbox.run("stop-local.sh", &[("SCANNERS", scanner), (status, "2")]);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("exit 2"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(sandbox.text("killed").is_empty());
    }
}

#[test]
fn valid_ledger_skips_port_scanning() {
    let sandbox = Sandbox::new();
    sandbox.ledger("frontend.pid", 42);
    sandbox.ledger("backend.pid", 43);
    let output = sandbox.run("stop-local.sh", &[("LSOF_STATUS", "2")]);
    assert_success(&output);
    assert!(sandbox.text("scan").is_empty());
    assert_eq!(sandbox.text("killed"), "42\n43\n");
}

#[test]
fn port_fallback_stops_only_the_matching_worktree_and_service() {
    for (name, value) in [
        ("FRONT_CWD", "/other-worktree/frontend"),
        ("FRONT_NAME", "backend"),
        ("FRONT_COMMAND", "trunk serve --port 80810"),
    ] {
        let sandbox = Sandbox::new();
        fs::write(sandbox.root.join("alive-42"), "").unwrap();
        let output = sandbox.run("stop-local.sh", &[("FRONT_SCAN_PIDS", "42"), (name, value)]);
        assert_success(&output);
        assert!(sandbox.text("killed").is_empty(), "{name}={value}");
    }
    let sandbox = Sandbox::new();
    fs::write(sandbox.root.join("alive-42"), "").unwrap();
    let output = sandbox.run("stop-local.sh", &[("FRONT_SCAN_PIDS", "42")]);
    assert_success(&output);
    assert_eq!(sandbox.text("killed"), "42\n");
}

#[test]
fn ledger_cannot_stop_a_server_from_another_worktree() {
    let sandbox = Sandbox::new();
    sandbox.ledger("frontend.pid", 42);
    let output = sandbox.run(
        "stop-local.sh",
        &[("FRONT_CWD", "/other-worktree/frontend")],
    );
    assert_success(&output);
    assert!(sandbox.text("killed").is_empty());
}

#[test]
fn scanner_fallback_and_each_successful_scanner_keep_owned_pids() {
    for env in [
        vec![("SCANNERS", "lsof ss"), ("LSOF_STATUS", "2")],
        vec![("SCANNERS", "ss")],
        vec![("SCANNERS", "fuser")],
    ] {
        let sandbox = Sandbox::new();
        fs::write(sandbox.root.join("alive-42"), "").unwrap();
        let mut env = env;
        env.push(("FRONT_SCAN_PIDS", "42"));
        let output = sandbox.run("stop-local.sh", &env);
        assert_success(&output);
        assert_eq!(sandbox.text("killed"), "42\n");
    }
}

#[test]
fn backend_fallback_rejects_foreign_worktrees_and_make_wrappers() {
    for (name, value) in [
        ("BACK_CWD", "/other-worktree/backend"),
        ("BACK_NAME", "make"),
    ] {
        let sandbox = Sandbox::new();
        fs::write(sandbox.root.join("alive-43"), "").unwrap();
        let output = sandbox.run("stop-local.sh", &[("BACK_SCAN_PIDS", "43"), (name, value)]);
        assert_success(&output);
        assert!(sandbox.text("killed").is_empty());
    }
    let sandbox = Sandbox::new();
    fs::write(sandbox.root.join("alive-43"), "").unwrap();
    let output = sandbox.run("stop-local.sh", &[("BACK_SCAN_PIDS", "43")]);
    assert_success(&output);
    assert_eq!(sandbox.text("killed"), "43\n");
}

#[test]
fn cargo_run_owner_can_be_stopped_during_compilation() {
    let sandbox = Sandbox::new();
    sandbox.ledger("backend.pid", 43);
    let output = sandbox.run(
        "stop-local.sh",
        &[
            ("BACK_NAME", "cargo"),
            ("BACK_COMMAND", "cargo run --bin backend"),
        ],
    );
    assert_success(&output);
    assert_eq!(sandbox.text("killed"), "43\n");
    assert_eq!(sandbox.text("scan").lines().count(), 1);
}

#[test]
fn missing_scanner_is_an_error_when_there_is_no_valid_ledger() {
    let sandbox = Sandbox::new();
    let output = sandbox.run("stop-local.sh", &[("SCANNERS", " ")]);
    assert!(!output.status.success());
    assert!(sandbox.text("killed").is_empty());
}

fn remove_tmp_files_containing(marker: &str) {
    if let Ok(entries) = fs::read_dir("/tmp") {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().contains(marker) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[test]
fn start_uses_one_backend_port_for_listener_url_proxy_and_pid() {
    let sandbox = Sandbox::new();
    let base = sandbox
        .root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output = sandbox.run(
        "start-local.sh",
        &[("CURL_STATUS", "0"), ("BACKEND_PORT", "4010")],
    );
    remove_tmp_files_containing(&base);
    assert_success(&output);

    let cargo = sandbox.text("cargo-env");
    assert!(cargo.contains("PORT=4010"), "{cargo}");
    assert!(
        cargo.contains("BACKEND_URL=http://127.0.0.1:4010"),
        "{cargo}"
    );

    let calls = sandbox.text("curl-calls");
    assert!(calls.contains("http://127.0.0.1:4010/ready"), "{calls}");
    assert!(!calls.contains("/health"), "{calls}");

    let config = sandbox.text("trunk-config");
    assert!(
        config.contains("backend = \"http://127.0.0.1:4010/api/\""),
        "{config}"
    );

    let names = sandbox.text("backend-pid-names");
    let prefix = format!("shoken-backend-dev-{base}-");
    assert!(
        names
            .lines()
            .any(|n| n.starts_with(&prefix) && n.ends_with("-4010.pid")),
        "{names}"
    );
    assert!(
        names
            .lines()
            .any(|n| n.starts_with(&prefix) && n.ends_with("-4010.log")),
        "{names}"
    );
}

#[test]
fn start_preserves_explicit_url_port_and_log_overrides() {
    let sandbox = Sandbox::new();
    let base = sandbox
        .root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let log = sandbox.root.join("backend.log");
    let output = sandbox.run(
        "start-local.sh",
        &[
            ("CURL_STATUS", "0"),
            ("BACKEND_URL", "http://example.test:7777"),
            ("BACKEND_LOG", log.to_str().unwrap()),
            ("FRONTEND_PORT", "8090"),
        ],
    );
    remove_tmp_files_containing(&base);
    assert_success(&output);

    let cargo = sandbox.text("cargo-env");
    assert!(cargo.contains("PORT=7777"), "{cargo}");
    assert!(
        cargo.contains("BACKEND_URL=http://example.test:7777"),
        "{cargo}"
    );
    assert!(log.exists());

    let calls = sandbox.text("curl-calls");
    assert!(calls.contains("http://example.test:7777/ready"), "{calls}");
    assert!(calls.contains("http://127.0.0.1:8090/"), "{calls}");

    let args = sandbox.text("trunk-args");
    assert!(args.contains("--port 8090"), "{args}");
    let config = sandbox.text("trunk-config");
    assert!(
        config.contains("backend = \"http://example.test:7777/api/\""),
        "{config}"
    );
}

#[test]
fn start_defaults_keep_stock_trunk_config_and_3001() {
    let sandbox = Sandbox::new();
    let base = sandbox
        .root
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output = sandbox.run("start-local.sh", &[("CURL_STATUS", "0")]);
    remove_tmp_files_containing(&base);
    assert_success(&output);

    let cargo = sandbox.text("cargo-env");
    assert!(cargo.contains("PORT=3001"), "{cargo}");
    let calls = sandbox.text("curl-calls");
    assert!(calls.contains("http://127.0.0.1:3001/ready"), "{calls}");

    let args = sandbox.text("trunk-args");
    assert!(args.contains("Trunk.toml"), "{args}");
    assert!(!args.contains("Trunk.local."), "{args}");
}
