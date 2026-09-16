use std::{error::Error as _, fs, time::Duration};

use serde_json::json;
use tempfile::TempDir;

use super::*;

const CONFIG: &str = r#"active_provider = "opencode-go"

[providers.openai]
base_url = "https://api.openai.com/v1"
model = "gpt-4.1-mini"
credential = "openai"

[providers.opencode-go]
base_url = "https://opencode.ai/zen/go/v1"
model = "gpt-5.6-luna"
credential = "opencode"
first_chunk_timeout_seconds = 15
stream_idle_timeout_seconds = 45

[providers.opencode-go.headers]
x-opencode-session = "${session_id}"
x-client = "test"

[providers.opencode-go.request_parameters]
reasoning_effort = "none"
"#;

const CREDENTIALS: &str = r#"[credentials.openai]
api_key = "openai-test-key"

[credentials.opencode]
api_key = "opencode-test-key"
"#;

fn paths(temp: &TempDir) -> AgentConfigPaths {
    AgentConfigPaths::new(
        temp.path().join("config.toml"),
        temp.path().join("credentials.toml"),
    )
    .unwrap_or_else(|error| panic!("valid temporary paths: {error}"))
}

fn write_config(temp: &TempDir, config: &str, credentials: &str) -> AgentConfigPaths {
    let paths = paths(temp);
    fs::write(paths.config(), config).unwrap_or_else(|error| panic!("write config: {error}"));
    fs::write(paths.credentials(), credentials)
        .unwrap_or_else(|error| panic!("write credentials: {error}"));
    paths
}

#[test]
fn resolves_selected_provider_with_all_neutral_fields() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let loaded = load_from_paths(write_config(&temp, CONFIG, CREDENTIALS))
        .unwrap_or_else(|error| panic!("load config: {error}"));

    let provider = loaded
        .active_provider()
        .unwrap_or_else(|error| panic!("active: {error}"));
    assert_eq!(loaded.active_provider_name(), "opencode-go");
    assert_eq!(provider.model(), "gpt-5.6-luna");
    assert_eq!(provider.api_key(), "opencode-test-key");
    assert_eq!(provider.first_chunk_timeout(), Duration::from_secs(15));
    assert_eq!(provider.stream_idle_timeout(), Duration::from_secs(45));
    assert_eq!(
        provider.request_parameters(),
        &serde_json::Map::from_iter([(String::from("reasoning_effort"), json!("none"))])
    );
    let headers = provider
        .headers("session-7")
        .unwrap_or_else(|error| panic!("headers: {error}"));
    assert_eq!(headers["x-opencode-session"], "session-7");
}

#[cfg(feature = "rig")]
#[test]
fn caller_builds_named_agent_from_rig_connection_settings() -> Result<(), Box<dyn std::error::Error>>
{
    use rig::client::AgentClientExt as _;

    let temp = TempDir::new()?;
    let loaded = load_from_paths(write_config(&temp, CONFIG, CREDENTIALS))?;
    let provider = loaded.active_provider()?;
    let settings: rig::providers::openai::CompletionsClientBuilder =
        provider.rig_completions_client_builder("caller-session")?;
    let client = settings.build()?;
    let agent = client
        .agent(provider.model())
        .additional_params(Value::Object(provider.request_parameters().clone()))
        .name("caller-owned-agent")
        .preamble("Caller-owned instructions")
        .build();

    assert_eq!(agent.name(), Some("caller-owned-agent"));
    Ok(())
}

struct SnapshotAdapter;

struct Snapshot {
    model: String,
    api_key: String,
    session: String,
    timeout: Duration,
    parameters: Map<String, Value>,
}

impl ProviderAdapter for SnapshotAdapter {
    type Output = Snapshot;
    type Error = ConfigError;

    fn adapt(
        &self,
        provider: &ResolvedProvider,
        session_id: &str,
    ) -> Result<Self::Output, Self::Error> {
        let headers = provider.headers(session_id)?;
        Ok(Snapshot {
            model: provider.model().to_owned(),
            api_key: provider.api_key().to_owned(),
            session: headers["x-opencode-session"]
                .to_str()
                .unwrap_or_default()
                .to_owned(),
            timeout: provider.stream_idle_timeout(),
            parameters: provider.request_parameters().clone(),
        })
    }
}

#[test]
fn custom_adapter_receives_selected_provider_without_rig() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let loaded = load_from_paths(write_config(&temp, CONFIG, CREDENTIALS))
        .unwrap_or_else(|error| panic!("load config: {error}"));
    let snapshot = loaded
        .active_provider()
        .unwrap_or_else(|error| panic!("active: {error}"))
        .adapt(&SnapshotAdapter, "custom-session")
        .unwrap_or_else(|error| panic!("adapt provider: {error}"));

    assert_eq!(snapshot.model, "gpt-5.6-luna");
    assert_eq!(snapshot.api_key, "opencode-test-key");
    assert_eq!(snapshot.session, "custom-session");
    assert_eq!(snapshot.timeout, Duration::from_secs(45));
    assert_eq!(snapshot.parameters["reasoning_effort"], json!("none"));
}

#[test]
fn malformed_credentials_do_not_leak_secret_through_error_formats() {
    let secret = "do-not-disclose-this-key";
    for source in [
        format!("[credentials.test]\napi_key = {secret}"),
        format!("[credentials]\ntest = \"{secret}\""),
    ] {
        let error = parse_credentials_toml(&source)
            .err()
            .unwrap_or_else(|| panic!("credentials must fail"));
        assert!(!format!("{error}").contains(secret));
        assert!(!format!("{error:#}").contains(secret));
        assert!(!format!("{error:?}").contains(secret));
        assert!(error.source().is_none());
    }
}

#[test]
fn expanded_headers_are_sensitive() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let loaded = load_from_paths(write_config(&temp, CONFIG, CREDENTIALS))
        .unwrap_or_else(|error| panic!("load config: {error}"));
    let headers = loaded
        .active_provider()
        .unwrap_or_else(|error| panic!("active: {error}"))
        .headers("private-session")
        .unwrap_or_else(|error| panic!("headers: {error}"));
    assert!(headers["x-opencode-session"].is_sensitive());
    assert!(!format!("{headers:?}").contains("private-session"));
}

#[test]
fn headers_expand_every_session_placeholder_and_preserve_unknown_literals() {
    let config = CONFIG.replace(
        "x-client = \"test\"",
        "x-client = \"${session_id}-${session_id}-${unknown}\"",
    );
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let loaded = load_from_paths(write_config(&temp, &config, CREDENTIALS))
        .unwrap_or_else(|error| panic!("load config: {error}"));
    let headers = loaded
        .active_provider()
        .unwrap_or_else(|error| panic!("active: {error}"))
        .headers("session-9")
        .unwrap_or_else(|error| panic!("headers: {error}"));
    assert_eq!(headers["x-client"], "session-9-session-9-${unknown}");
}

#[test]
fn rejects_invalid_headers_and_case_insensitive_duplicates() {
    for replacement in [
        "bad header = \"ok\"",
        "x-client = \"line\\nbreak\"",
        "x-client = \"one\"\nX-Client = \"two\"",
    ] {
        let config = CONFIG.replace("x-client = \"test\"", replacement);
        let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
        assert!(load_from_paths(write_config(&temp, &config, CREDENTIALS)).is_err());
    }
}

#[test]
fn selects_each_named_provider_with_its_own_model_and_key() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let opencode = load_from_paths(write_config(&temp, CONFIG, CREDENTIALS))
        .unwrap_or_else(|error| panic!("load opencode: {error}"));
    let provider = opencode
        .active_provider()
        .unwrap_or_else(|error| panic!("active: {error}"));
    assert_eq!(
        (provider.model(), provider.api_key()),
        ("gpt-5.6-luna", "opencode-test-key")
    );

    let openai_config = CONFIG.replace(
        "active_provider = \"opencode-go\"",
        "active_provider = \"openai\"",
    );
    let openai = load_from_paths(write_config(&temp, &openai_config, CREDENTIALS))
        .unwrap_or_else(|error| panic!("load openai: {error}"));
    let provider = openai
        .active_provider()
        .unwrap_or_else(|error| panic!("active: {error}"));
    assert_eq!(
        (provider.model(), provider.api_key()),
        ("gpt-4.1-mini", "openai-test-key")
    );
}

#[test]
fn rejects_unknown_selection_missing_credentials_and_zero_timeouts() {
    for config in [
        CONFIG.replace(
            "active_provider = \"opencode-go\"",
            "active_provider = \"missing\"",
        ),
        CONFIG.replace("credential = \"opencode\"", "credential = \"missing\""),
        CONFIG.replace(
            "first_chunk_timeout_seconds = 15",
            "first_chunk_timeout_seconds = 0",
        ),
        CONFIG.replace(
            "stream_idle_timeout_seconds = 45",
            "stream_idle_timeout_seconds = 0",
        ),
    ] {
        let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
        assert!(load_from_paths(write_config(&temp, &config, CREDENTIALS)).is_err());
    }
}

#[test]
fn initialization_never_overwrites_existing_files_and_paths_are_strict() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let paths = paths(&temp);
    fs::write(paths.config(), CONFIG).unwrap_or_else(|error| panic!("write config: {error}"));
    fs::write(paths.credentials(), CREDENTIALS)
        .unwrap_or_else(|error| panic!("write credentials: {error}"));
    let loaded =
        load_or_initialize(paths.clone()).unwrap_or_else(|error| panic!("initialize: {error}"));
    assert!(!loaded.created_files());
    assert_eq!(
        fs::read_to_string(paths.config()).unwrap_or_default(),
        CONFIG
    );
    assert!(AgentConfigPaths::new("relative/config.toml", "/absolute/credentials.toml").is_err());
    assert!(AgentConfigPaths::new("/absolute/same.toml", "/absolute/same.toml").is_err());
}

#[test]
fn rejects_url_with_credentials_or_query() {
    let config = CONFIG.replace(
        "https://opencode.ai/zen/go/v1",
        "https://key@opencode.ai/v1?secret=value",
    );
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let error = load_from_paths(write_config(&temp, &config, CREDENTIALS))
        .err()
        .unwrap_or_else(|| panic!("URL must fail"));
    assert!(matches!(error, ConfigError::InvalidBaseUrl(name) if name == "opencode-go"));
}

#[test]
fn initialize_creates_private_credentials_file() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let paths = paths(&temp);
    let loaded =
        load_or_initialize(paths.clone()).unwrap_or_else(|error| panic!("initialize: {error}"));
    assert!(loaded.created_files());
    assert!(paths.config().exists());
    assert!(paths.credentials().exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            fs::metadata(paths.credentials())
                .unwrap_or_else(|error| panic!("metadata: {error}"))
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn initialize_supports_distinct_config_and_credentials_directories() {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let paths = AgentConfigPaths::new(
        temp.path().join("config-dir/config.toml"),
        temp.path().join("credentials-dir/credentials.toml"),
    )
    .unwrap_or_else(|error| panic!("paths: {error}"));

    load_or_initialize(paths.clone()).unwrap_or_else(|error| panic!("initialize: {error}"));

    assert!(paths.config().is_file());
    assert!(paths.credentials().is_file());
}
