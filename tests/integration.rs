use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

const TEST_MESSAGE: &str = "integration-test-hello-42";
const TIMEOUT: Duration = Duration::from_secs(30);

/// Extract the ticket from a line like `[HH:MM:SS] > ticket to join us: <ticket>`.
fn parse_ticket(line: &str) -> Option<&str> {
    let marker = "ticket to join us: ";
    line.find(marker)
        .map(|pos| line[pos + marker.len()..].trim())
}

// Requires network access to iroh relay servers (N0 presets).
// Run explicitly: cargo test -- --ignored
#[tokio::test]
#[ignore]
async fn open_send_join_receives() {
    let bin = std::env::var("CARGO_BIN_EXE_iroh_message")
        .unwrap_or_else(|_| "target/debug/iroh-message".to_string());

    // 1. Spawn the "open" node (--name is a top-level flag, before the subcommand)
    let mut open = Command::new(&bin)
        .args(["--name", "alice", "open"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn open node");

    let mut open_stdin = open.stdin.take().unwrap();
    let open_stdout = open.stdout.take().unwrap();
    let mut open_lines = BufReader::new(open_stdout).lines();

    // 2. Read stdout until we get the ticket
    let ticket = tokio::time::timeout(TIMEOUT, async {
        loop {
            match open_lines.next_line().await {
                Ok(Some(line)) => {
                    eprintln!("[open] {line}");
                    if let Some(t) = parse_ticket(&line) {
                        return t.to_string();
                    }
                }
                Ok(None) => panic!("open stdout closed before printing ticket"),
                Err(e) => panic!("error reading open stdout: {e}"),
            }
        }
    })
    .await
    .expect("timeout waiting for ticket from open node");

    // Drain remaining open stdout in background to prevent pipe blocking
    tokio::spawn(async move {
        while let Ok(Some(_)) = open_lines.next_line().await {}
    });

    // 3. Spawn the "join" node with the ticket
    let mut join = Command::new(&bin)
        .args(["--name", "bob", "join", &ticket])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn join node");

    let join_stdout = join.stdout.take().unwrap();
    let mut join_lines = BufReader::new(join_stdout).lines();

    // Wait for the join node to report "connected!"
    tokio::time::timeout(TIMEOUT, async {
        loop {
            match join_lines.next_line().await {
                Ok(Some(line)) => {
                    eprintln!("[join] {line}");
                    if line.contains("> connected!") {
                        return;
                    }
                }
                Ok(None) => panic!("join stdout closed before connecting"),
                Err(e) => panic!("error reading join stdout: {e}"),
            }
        }
    })
    .await
    .expect("timeout waiting for join node to connect");

    // Brief pause to let the gossip mesh stabilize
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 4. Send a test message from the open node's stdin
    open_stdin
        .write_all(format!("{TEST_MESSAGE}\n").as_bytes())
        .await
        .expect("failed to write to open stdin");
    open_stdin.flush().await.expect("failed to flush open stdin");

    // 5. Read join node's stdout until we see the test message
    let received_line = tokio::time::timeout(TIMEOUT, async {
        loop {
            match join_lines.next_line().await {
                Ok(Some(line)) => {
                    eprintln!("[join] {line}");
                    if line.contains(TEST_MESSAGE) {
                        return line;
                    }
                }
                Ok(None) => panic!("join stdout closed before receiving message"),
                Err(e) => panic!("error reading join stdout: {e}"),
            }
        }
    })
    .await
    .expect("timeout waiting for message on join node");

    // 6. Assert the message arrived
    assert!(
        received_line.contains(TEST_MESSAGE),
        "expected join node to receive message containing '{TEST_MESSAGE}', got: {received_line}"
    );

    // Clean up child processes
    let _ = open.kill().await;
    let _ = join.kill().await;
}
