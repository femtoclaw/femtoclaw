use femtoclaw::{Agent, Config};
use serde_json::json;

#[tokio::test]
async fn test_agent_echo_brain() {
    let agent = Agent::new(Config::default()).expect("agent created");
    let response = agent.run("hello world").await.expect("run succeeded");
    assert!(response.contains("ACK: hello world"));
}

#[tokio::test]
async fn test_agent_memory_persistence() {
    let agent = Agent::new(Config::default()).expect("agent created");
    agent.run("first").await.unwrap();
    agent.run("second").await.unwrap();
    let history = agent.history().await;
    // Each run adds user + assistant => 4 messages total
    assert!(history.len() >= 4);
}

#[tokio::test]
async fn test_agent_reset_clears_memory() {
    let agent = Agent::new(Config::default()).expect("agent failed");
    agent.run("something").await.unwrap();
    agent.reset().await;
    let history = agent.history().await;
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_execute_tool_shell_allowed() {
    let agent = Agent::new(Config::default()).expect("agent failed");
    let result = agent.execute_tool("shell", json!({"bin":"echo","argv":["test123"]})).await.unwrap();
    assert!(result.contains("test123"));
}

#[tokio::test]
async fn test_execute_tool_unknown_denied() {
    let agent = Agent::new(Config::default()).expect("agent failed");
    let result = agent.execute_tool("unknown_cap", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_tool_fs() {
    let agent = Agent::new(Config::default()).unwrap();
    let result = agent.execute_tool("fs", json!({"path": "/nonexistent/path/to/file"})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_execute_tool_process() {
    let agent = Agent::new(Config::default()).unwrap();
    let result = agent.execute_tool("process", json!({"program":"echo","args":["hi"]})).await.unwrap();
    assert!(result.contains("hi"));
}

#[tokio::test]
async fn test_execute_tool_net() {
    let agent = Agent::new(Config::default()).unwrap();
    let result = agent.execute_tool("net", json!({"url": "http://example.com"})).await.unwrap();
    assert!(result.contains("http://example.com"));
}
