use femtoclaw::{Agent, Config};
use femtoclaw::brain::{BrainKind, mock::MockBrain};

#[tokio::test]
async fn test_autonomous_loop_completion() {
    let responses = vec![
        r#"{"tool_call":{"tool":"shell","args":{"bin":"echo","argv":["step 1"]}}}"#.to_string(),
        r#"{"message":{"content":"Task complete"}}"#.to_string(),
    ];
    let mock_brain = MockBrain::new(responses);
    
    let agent = Agent::new(Config::default())
        .expect("agent created")
        .with_brain(BrainKind::Mock(mock_brain));
        
    let response = agent.run("do multi-step").await.expect("run succeeded");
    
    assert_eq!(response, "Task complete");
    
    let history = agent.history().await;
    // Expected history:
    // 1. User: do multi-step
    // 2. Assistant: {"tool_call":...}
    // 3. Tool: result of echo step 1
    // 4. Assistant: Task complete
    assert_eq!(history.len(), 4);
    assert_eq!(history[0].role, femtoclaw::Role::User);
    assert_eq!(history[1].role, femtoclaw::Role::Assistant);
    assert_eq!(history[2].role, femtoclaw::Role::Tool);
    assert_eq!(history[3].role, femtoclaw::Role::Assistant);
}

#[tokio::test]
async fn test_autonomous_loop_max_iterations() {
    // Brain that always calls a tool and never finishes
    let responses = vec![
        r#"{"tool_call":{"tool":"shell","args":{"bin":"echo","argv":["looping"]}}}"#.to_string(); 20
    ];
    let mock_brain = MockBrain::new(responses);
    
    let mut config = Config::default();
    config.max_iterations = 3;
    
    let agent = Agent::new(config)
        .expect("agent created")
        .with_brain(BrainKind::Mock(mock_brain));
        
    let result = agent.run("infinite loop").await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Maximum iterations reached"));
}
