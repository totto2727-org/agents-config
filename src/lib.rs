//! Shared, deterministic OpenAI-compatible agent configuration.
//!
//! This crate reads only the explicit paths supplied by callers. [`AgentConfigPaths::standard`]
//! selects `~/.agents/config.toml` (or `$AGENTS_CONFIG`) and never searches the current directory.

use std::{
    collections::BTreeMap,
    env, fs,
    io::Write as _,
    path::{Path, PathBuf},
    time::Duration,
};

use http::{HeaderMap, HeaderValue, Uri, header::HeaderName};
use serde::Deserialize;
use serde_json::{Map, Value};
use tempfile::NamedTempFile;
use thiserror::Error;

pub const DEFAULT_CONFIG: &str = r#"active_provider = "openai"

[providers.openai]
base_url = "https://api.openai.com/v1"
model = "gpt-4.1-mini"
credential = "openai"
first_chunk_timeout_seconds = 30
stream_idle_timeout_seconds = 60
"#;

pub const DEFAULT_CREDENTIALS: &str = r#"[credentials.openai]
api_key = "replace-me"
"#;

const DEFAULT_FIRST_CHUNK_TIMEOUT_SECONDS: u64 = 30;
const DEFAULT_STREAM_IDLE_TIMEOUT_SECONDS: u64 = 60;

/// The two files that define an agent configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentConfigPaths {
    config: PathBuf,
    credentials: PathBuf,
}

impl AgentConfigPaths {
    ///
    /// # Errors
    /// Returns an error if either path is relative or both paths are equal.
    pub fn new(
        config: impl Into<PathBuf>,
        credentials: impl Into<PathBuf>,
    ) -> Result<Self, ConfigError> {
        let config = config.into();
        let credentials = credentials.into();
        if !config.is_absolute() || !credentials.is_absolute() {
            return Err(ConfigError::InvalidPath(
                "config and credentials paths must be absolute".into(),
            ));
        }
        if config == credentials {
            return Err(ConfigError::InvalidPath(
                "config and credentials paths must differ".into(),
            ));
        }
        Ok(Self {
            config,
            credentials,
        })
    }

    /// Resolve the desktop-wide paths without inspecting the current directory.
    ///
    /// `$AGENTS_CONFIG` may override the config path for tests and controlled deployments.
    /// Credentials remain beside that selected config file.
    ///
    /// # Errors
    /// Returns an error when neither `AGENTS_CONFIG` nor `HOME` supplies an absolute path.
    pub fn standard() -> Result<Self, ConfigError> {
        let config = match env::var_os("AGENTS_CONFIG") {
            Some(path) => PathBuf::from(path),
            None => home_dir()?.join(".agents").join("config.toml"),
        };
        let credentials = config
            .parent()
            .ok_or_else(|| ConfigError::InvalidPath("config path has no parent directory".into()))?
            .join("credentials.toml");
        Self::new(config, credentials)
    }

    #[must_use]
    pub fn config(&self) -> &Path {
        &self.config
    }

    #[must_use]
    pub fn config_path(&self) -> &Path {
        self.config()
    }

    #[must_use]
    pub fn credentials(&self) -> &Path {
        &self.credentials
    }

    #[must_use]
    pub fn credentials_path(&self) -> &Path {
        self.credentials()
    }
}

/// A loaded configuration and the files created while initializing it.
pub struct LoadedAgentsConfig {
    providers: BTreeMap<String, ResolvedProvider>,
    active_provider: String,
    paths: AgentConfigPaths,
    created_files: bool,
}

impl LoadedAgentsConfig {
    #[must_use]
    pub fn paths(&self) -> &AgentConfigPaths {
        &self.paths
    }

    #[must_use]
    pub fn created_files(&self) -> bool {
        self.created_files
    }

    #[must_use]
    pub fn active_provider_name(&self) -> &str {
        &self.active_provider
    }

    /// # Errors
    /// Returns an error if the configured active provider is absent.
    pub fn active_provider(&self) -> Result<&ResolvedProvider, ConfigError> {
        self.provider(&self.active_provider)
    }

    /// # Errors
    /// Returns an error if `name` is not configured.
    pub fn provider(&self, name: &str) -> Result<&ResolvedProvider, ConfigError> {
        self.providers
            .get(name)
            .ok_or_else(|| ConfigError::UnknownProvider(name.to_owned()))
    }

    pub fn providers(&self) -> impl Iterator<Item = (&str, &ResolvedProvider)> {
        self.providers
            .iter()
            .map(|(name, provider)| (name.as_str(), provider))
    }
}

/// A validated provider selected from configuration and credentials.
///
/// This deliberately does not implement `Debug`, preventing accidental secret logging.
pub struct ResolvedProvider {
    name: String,
    base_url: Uri,
    model: String,
    credential_name: String,
    api_key: Secret,
    headers: BTreeMap<String, String>,
    request_parameters: Map<String, Value>,
    first_chunk_timeout: Duration,
    stream_idle_timeout: Duration,
}

impl ResolvedProvider {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn base_url(&self) -> &Uri {
        &self.base_url
    }

    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    #[must_use]
    pub fn credential_name(&self) -> &str {
        &self.credential_name
    }

    /// Return the API key only to configure an outbound client. Never log this value.
    #[must_use]
    pub fn api_key(&self) -> &str {
        self.api_key.expose()
    }

    #[must_use]
    pub fn request_parameters(&self) -> &Map<String, Value> {
        &self.request_parameters
    }

    #[must_use]
    pub fn first_chunk_timeout(&self) -> Duration {
        self.first_chunk_timeout
    }

    #[must_use]
    pub fn stream_idle_timeout(&self) -> Duration {
        self.stream_idle_timeout
    }

    /// Expand `${session_id}` in configured values and validate HTTP headers.
    /// With `None`, omit only headers containing that placeholder.
    ///
    /// # Errors
    /// Returns an error if expansion cannot form a valid HTTP header.
    pub fn headers(&self, session_id: Option<&str>) -> Result<HeaderMap, ConfigError> {
        let mut headers = HeaderMap::new();
        for (name, template) in &self.headers {
            let name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| ConfigError::InvalidHeaderName(name.clone()))?;
            let expanded = match session_id {
                Some(id) => template.replace("${session_id}", id),
                None if template.contains("${session_id}") => continue,
                None => template.clone(),
            };
            let mut value = HeaderValue::from_str(&expanded)
                .map_err(|_| ConfigError::InvalidHeaderValue(name.to_string()))?;
            value.set_sensitive(true);
            if headers.insert(name.clone(), value).is_some() {
                return Err(ConfigError::DuplicateHeader(name.to_string()));
            }
        }
        Ok(headers)
    }

    /// # Errors
    /// Propagates the adapter's conversion error.
    pub fn adapt<A>(&self, adapter: &A, session_id: Option<&str>) -> Result<A::Output, A::Error>
    where
        A: ProviderAdapter,
    {
        adapter.adapt(self, session_id)
    }
}

/// Converts a resolved provider to an application-specific client or request type.
pub trait ProviderAdapter {
    type Output;
    type Error;

    /// # Errors
    /// Returns an implementation-defined conversion error.
    fn adapt(
        &self,
        provider: &ResolvedProvider,
        session_id: Option<&str>,
    ) -> Result<Self::Output, Self::Error>;
}

/// Errors omit configuration source and all credential or header values.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[cfg(feature = "rig")]
    #[error("failed to build the Rig client")]
    RigClientBuild,
    #[error("HOME is unavailable and AGENTS_CONFIG is not set")]
    HomeUnavailable,
    #[error("invalid configuration path: {0}")]
    InvalidPath(String),
    #[error("failed to create configuration directory")]
    CreateDirectory(#[source] std::io::Error),
    #[error("failed to create configuration file")]
    CreateFile(#[source] std::io::Error),
    #[error("failed to write configuration file")]
    WriteFile(#[source] std::io::Error),
    #[error("failed to read configuration file")]
    ReadConfig(#[source] std::io::Error),
    #[error("failed to read credentials file")]
    ReadCredentials(#[source] std::io::Error),
    #[error("config.toml is invalid")]
    ConfigToml,
    #[error("credentials.toml is invalid")]
    CredentialsToml,
    #[error("active provider '{0}' is not configured")]
    UnknownProvider(String),
    #[error("provider '{0}' has an invalid base_url")]
    InvalidBaseUrl(String),
    #[error("provider '{0}' requires a non-empty {1}")]
    EmptyProviderField(String, &'static str),
    #[error("provider '{0}' has a zero {1}")]
    ZeroTimeout(String, &'static str),
    #[error("credential '{0}' is not configured")]
    UnknownCredential(String),
    #[error("credential '{0}' has an empty api_key")]
    EmptyApiKey(String),
    #[error("invalid provider header name '{0}'")]
    InvalidHeaderName(String),
    #[error("invalid provider header value for '{0}'")]
    InvalidHeaderValue(String),
    #[error("duplicate provider header name '{0}' (case-insensitive)")]
    DuplicateHeader(String),
}

/// Parse configuration TOML independently of filesystem loading.
///
/// # Errors
/// Returns an error if TOML syntax or configuration structure is invalid.
pub fn parse_config_toml(source: &str) -> Result<Config, ConfigError> {
    toml::from_str(source).map_err(|_| ConfigError::ConfigToml)
}

/// Load existing files only. No current-directory or project-specific fallback is used.
///
/// # Errors
/// Returns an error if files cannot be read, are unsafe to prepare, or fail validation.
pub fn load_from_paths(paths: AgentConfigPaths) -> Result<LoadedAgentsConfig, ConfigError> {
    set_credentials_permissions(paths.credentials())?;
    let config_source = fs::read_to_string(paths.config()).map_err(ConfigError::ReadConfig)?;
    let credentials_source =
        fs::read_to_string(paths.credentials()).map_err(ConfigError::ReadCredentials)?;
    resolve(
        parse_config_toml(&config_source)?,
        &parse_credentials_toml(&credentials_source)?,
        paths,
        false,
    )
}

/// Create missing standard files, enforce credentials mode `0600` on Unix, then load them.
///
/// # Errors
/// Returns an error if directories or files cannot be prepared, read, or validated.
pub fn load_or_initialize(paths: AgentConfigPaths) -> Result<LoadedAgentsConfig, ConfigError> {
    let config_directory = paths
        .config()
        .parent()
        .ok_or_else(|| ConfigError::InvalidPath("config path has no parent directory".into()))?;
    let credentials_directory = paths.credentials().parent().ok_or_else(|| {
        ConfigError::InvalidPath("credentials path has no parent directory".into())
    })?;
    fs::create_dir_all(config_directory).map_err(ConfigError::CreateDirectory)?;
    fs::create_dir_all(credentials_directory).map_err(ConfigError::CreateDirectory)?;
    let credentials_created = create_if_missing(paths.credentials(), DEFAULT_CREDENTIALS, true)?;
    let config_created = create_if_missing(paths.config(), DEFAULT_CONFIG, false)?;
    set_credentials_permissions(paths.credentials())?;
    let mut loaded = load_from_paths(paths)?;
    loaded.created_files = config_created || credentials_created;
    Ok(loaded)
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub active_provider: String,
    pub providers: BTreeMap<String, ProviderConfig>,
}

#[derive(Clone, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
    pub model: String,
    pub credential: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub request_parameters: Map<String, Value>,
    #[serde(default = "default_first_chunk_timeout")]
    pub first_chunk_timeout_seconds: u64,
    #[serde(default = "default_stream_idle_timeout")]
    pub stream_idle_timeout_seconds: u64,
}

#[derive(Deserialize)]
struct CredentialsFile {
    credentials: BTreeMap<String, Credential>,
}

#[derive(Deserialize)]
struct Credential {
    api_key: String,
}

struct Secret(String);

impl Secret {
    fn expose(&self) -> &str {
        &self.0
    }
}

fn parse_credentials_toml(source: &str) -> Result<CredentialsFile, ConfigError> {
    toml::from_str(source).map_err(|_| ConfigError::CredentialsToml)
}

fn resolve(
    config: Config,
    credentials: &CredentialsFile,
    paths: AgentConfigPaths,
    created_files: bool,
) -> Result<LoadedAgentsConfig, ConfigError> {
    let mut providers = BTreeMap::new();
    for (name, provider) in config.providers {
        let resolved = resolve_provider(&name, provider, credentials)?;
        providers.insert(name, resolved);
    }
    if !providers.contains_key(&config.active_provider) {
        return Err(ConfigError::UnknownProvider(config.active_provider));
    }
    Ok(LoadedAgentsConfig {
        providers,
        active_provider: config.active_provider,
        paths,
        created_files,
    })
}

fn resolve_provider(
    name: &str,
    provider: ProviderConfig,
    credentials: &CredentialsFile,
) -> Result<ResolvedProvider, ConfigError> {
    require_non_empty(name, "base_url", &provider.base_url)?;
    require_non_empty(name, "model", &provider.model)?;
    require_non_empty(name, "credential", &provider.credential)?;
    if provider.first_chunk_timeout_seconds == 0 {
        return Err(ConfigError::ZeroTimeout(
            name.to_owned(),
            "first_chunk_timeout_seconds",
        ));
    }
    if provider.stream_idle_timeout_seconds == 0 {
        return Err(ConfigError::ZeroTimeout(
            name.to_owned(),
            "stream_idle_timeout_seconds",
        ));
    }
    let base_url: Uri = provider
        .base_url
        .parse()
        .map_err(|_| ConfigError::InvalidBaseUrl(name.to_owned()))?;
    if !matches!(base_url.scheme_str(), Some("http" | "https"))
        || base_url.authority().is_none()
        || base_url
            .authority()
            .is_some_and(|authority| authority.as_str().contains('@'))
        || base_url.query().is_some()
    {
        return Err(ConfigError::InvalidBaseUrl(name.to_owned()));
    }
    let credential = credentials
        .credentials
        .get(&provider.credential)
        .ok_or_else(|| ConfigError::UnknownCredential(provider.credential.clone()))?;
    if credential.api_key.trim().is_empty() {
        return Err(ConfigError::EmptyApiKey(provider.credential));
    }
    validate_header_names(&provider.headers)?;
    Ok(ResolvedProvider {
        name: name.to_owned(),
        base_url,
        model: provider.model,
        credential_name: provider.credential,
        api_key: Secret(credential.api_key.clone()),
        headers: provider.headers,
        request_parameters: provider.request_parameters,
        first_chunk_timeout: Duration::from_secs(provider.first_chunk_timeout_seconds),
        stream_idle_timeout: Duration::from_secs(provider.stream_idle_timeout_seconds),
    })
}

fn require_non_empty(name: &str, field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        Err(ConfigError::EmptyProviderField(name.to_owned(), field))
    } else {
        Ok(())
    }
}

fn validate_header_names(headers: &BTreeMap<String, String>) -> Result<(), ConfigError> {
    let mut seen = std::collections::BTreeSet::new();
    for name in headers.keys() {
        HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| ConfigError::InvalidHeaderName(name.clone()))?;
        let normalized = name.to_ascii_lowercase();
        if !seen.insert(normalized) {
            return Err(ConfigError::DuplicateHeader(name.clone()));
        }
        HeaderValue::from_str(&headers[name])
            .map_err(|_| ConfigError::InvalidHeaderValue(name.clone()))?;
    }
    Ok(())
}

fn create_if_missing(path: &Path, content: &str, credentials: bool) -> Result<bool, ConfigError> {
    match fs::symlink_metadata(path) {
        Ok(_) => return Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(ConfigError::CreateFile(error)),
    }
    let parent = path
        .parent()
        .ok_or_else(|| ConfigError::InvalidPath("file path has no parent directory".into()))?;
    let mut file = NamedTempFile::new_in(parent).map_err(ConfigError::CreateFile)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        file.as_file_mut()
            .set_permissions(fs::Permissions::from_mode(if credentials {
                0o600
            } else {
                0o644
            }))
            .map_err(ConfigError::CreateFile)?;
    }
    file.write_all(content.as_bytes())
        .map_err(ConfigError::WriteFile)?;
    match file.persist_noclobber(path) {
        Ok(_) => Ok(true),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(ConfigError::CreateFile(error.error)),
    }
}

fn set_credentials_permissions(path: &Path) -> Result<(), ConfigError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(ConfigError::CreateFile)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn home_dir() -> Result<PathBuf, ConfigError> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or(ConfigError::HomeUnavailable)
}

const fn default_first_chunk_timeout() -> u64 {
    DEFAULT_FIRST_CHUNK_TIMEOUT_SECONDS
}
const fn default_stream_idle_timeout() -> u64 {
    DEFAULT_STREAM_IDLE_TIMEOUT_SECONDS
}

#[cfg(feature = "rig")]
impl ResolvedProvider {
    /// Convert provider settings to a Rig agent builder without building the agent.
    ///
    /// Applies the connection, model, and additional request parameters.
    /// The optional session ID only expands headers; `None` omits session headers.
    /// Callers add prompts and tools, then build the agent and own runtime policies.
    ///
    /// # Errors
    /// Returns an error if headers are invalid or the underlying client cannot be built.
    pub fn rig_agent_builder(
        &self,
        session_id: Option<&str>,
    ) -> Result<rig::agent::AgentBuilder<rig::providers::openai::CompletionModel>, ConfigError>
    {
        use rig::client::AgentClientExt as _;

        let client = rig::providers::openai::CompletionsClient::builder()
            .api_key(self.api_key())
            .base_url(self.base_url.to_string())
            .http_headers(self.headers(session_id)?)
            .build()
            .map_err(|_| ConfigError::RigClientBuild)?;
        let builder = client.agent(self.model());
        Ok(if self.request_parameters.is_empty() {
            builder
        } else {
            builder.additional_params(Value::Object(self.request_parameters.clone()))
        })
    }
}

#[cfg(test)]
mod tests;
