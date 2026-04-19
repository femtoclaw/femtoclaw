use femtoclaw::{Agent, Config};
use serde_json::json;

fn config_with_capabilities(capabilities: &[&str]) -> Config {
    let mut config = Config::default();
    config.allowed_capabilities = capabilities.iter().map(|cap| cap.to_string()).collect();
    config
}

#[tokio::test]
async fn test_memory_eviction() {
    let max = 3usize;
    let config = Config {
        brain: femtoclaw::config::BrainConfig {
            backend: "echo".to_string(),
            model: None,
            api_key: None,
        },
        max_memory: max,
        max_iterations: 10,
        allowed_capabilities: vec![],
    };
    let agent = Agent::new(config).expect("agent failed");

    // Send more than max messages
    for i in 0..max + 2 {
        agent.run(&format!("msg {}", i)).await.unwrap();
    }

    let history = agent.history().await;
    assert!(
        history.len() <= max,
        "history len {} exceeds max {}",
        history.len(),
        max
    );
}

#[tokio::test]
async fn test_shell_tool_with_allowlist() {
    let agent = Agent::new(config_with_capabilities(&["shell"])).unwrap();
    // The default allowlist includes 'echo'
    let result = agent
        .execute_tool("shell", json!({"bin":"echo","argv":["hello"]}))
        .await
        .unwrap();
    assert!(result.contains("hello"));
}

#[tokio::test]
async fn test_shell_tool_not_allowlisted() {
    let agent = Agent::new(config_with_capabilities(&["shell"])).unwrap();
    // Use a command that is not allowlisted (by default)
    // The default allowlist: ls, cat, pwd, whoami, git, echo, head, tail, wc, grep, powershell*, notepad*
    // Let's try "false" (a command that exists on Linux but is not allowlisted)
    let result = agent
        .execute_tool("shell", json!({"bin":"false","argv":[]}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not allowed") || err.contains("denied"));
}

#[tokio::test]
async fn test_web_get_tool_basic() {
    let agent = Agent::new(config_with_capabilities(&["web.get"])).unwrap();
    // We'll fetch a small site (example.com is redirect; maybe use httpbin)
    // Use a test-friendly URL: https://httpbin.org/status/200
    // Note: This test requires network and may be flaky; allow failure if offline.
    let result = agent
        .execute_tool("web.get", json!({"url":"https://httpbin.org/status/200"}))
        .await;
    // If network available, expect STATUS: 200; else error.
    match result {
        Ok(out) => assert!(out.contains("STATUS: 200")),
        Err(e) => {
            let e = e.to_string();
            // In CI/offline, this is acceptable.
            assert!(e.contains("failed") || e.contains("timeout") || e.contains("resolve"));
        }
    }
}
