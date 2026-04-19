//! Runtime Configuration.

#[derive(Debug, Clone)]
pub struct Config {
    pub brain: BrainConfig,
    pub max_memory: usize,
    pub max_iterations: usize,
    pub allowed_capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BrainConfig {
    pub backend: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            brain: BrainConfig {
                backend: std::env::var("FEMTO_BRAIN").unwrap_or_else(|_| "echo".to_string()),
                model: std::env::var("FEMTO_OPENAI_MODEL").ok(),
                api_key: std::env::var("FEMTO_OPENAI_API_KEY").ok(),
            },
            max_memory: 1000,
            max_iterations: 10,
            allowed_capabilities: parse_allowed_capabilities(),
        }
    }
}

fn parse_allowed_capabilities() -> Vec<String> {
    std::env::var("FEMTO_ALLOWED_CAPABILITIES")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|cap| !cap.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}
