# agents-config

`agents-config` lets independent applications share named OpenAI-compatible providers and credentials, then convert validated settings into their own library types.
It has no dependency on GlossShift or GPUI, and Rig support is optional.

## Usage

Use the same configured provider for an assistant in another Rust application:

```rust
use agents_config::{AgentConfigPaths, load_from_paths};
use rig::completion::Prompt;

async fn summarize() -> Result<String, Box<dyn std::error::Error>> {
    let config = load_from_paths(AgentConfigPaths::standard()?)?;
    let provider = config.provider("opencode-go")?;
    let agent = provider.rig_agent_builder(None)?
        .preamble("Summarize in one sentence.")
        .build();
    Ok(agent.prompt("The release adds shared agent configuration and reusable provider adapters.").await?)
}
```

The crate applies connection settings, model, and additional request parameters to a Rig `AgentBuilder`, without building the agent.
The application adds prompts and tools, calls `.build()`, and owns runtime behavior.
Call `active_provider()` instead of `provider("opencode-go")` to follow the shared default.
A session ID is optional and only expands `${session_id}` in custom headers.
Pass `None` to omit headers containing that placeholder, or `Some("summary-session-1")` when the provider needs them.
Fixed headers remain unchanged; this crate never generates session IDs.
Never log the exposed API key, request parameters containing secrets, or raw configuration content.

## Key features

- Multiple named providers and a shared `active_provider` selection.
- OpenAI Chat Completions-compatible URLs, models, named API keys, custom headers, and additional JSON request fields.
- `${session_id}` header expansion without environment-variable interpolation.
- Explicit paths or desktop-wide `~/.agents` defaults without current-directory discovery.
- Credential-file permissions of `0600` on Unix.
- A library-neutral `ResolvedProvider` and `ProviderAdapter` interface, with optional Rig 0.41 conversion.

## Prerequisites

- A Rust application using edition 2024 or a compatible toolchain.
- Credentials and a model supported by an OpenAI-compatible Chat Completions service.

## Setup

This crate is currently an independent local repository in the virtual monorepo, not a published registry package.
From an application under `app/<name>/`, add:

```toml
[dependencies]
agents-config = { path = "../../package/agents-config", features = ["rig"] }
rig = "0.41"
```

Omit the `rig` feature and the direct `rig` dependency when implementing an adapter for another library.

## Configuration

By default, configuration comes from `~/.agents/config.toml`, with keys in `~/.agents/credentials.toml`.
`AGENTS_CONFIG=/absolute/path/config.toml` overrides the configuration path and selects its sibling `credentials.toml`.
The loader never searches the working directory, so desktop and command-line consumers do not accidentally use different project settings.

```toml
# ~/.agents/config.toml
active_provider = "opencode-go"

[providers.openai]
base_url = "https://api.openai.com/v1"
model = "gpt-4.1-mini"
credential = "openai"

[providers.opencode-go]
base_url = "https://opencode.ai/zen/go/v1"
model = "kimi-k2.5"
credential = "opencode"
first_chunk_timeout_seconds = 30
stream_idle_timeout_seconds = 60

[providers.opencode-go.headers]
x-opencode-session = "${session_id}"
User-Agent = "my-assistant/0.1.0"

# Optional fields supported by your selected provider and model:
# [providers.opencode-go.request_parameters]
# reasoning_effort = "none"
```

Choose a model available to your account.
The URL must use HTTP or HTTPS and include the API prefix, such as `/v1` or `/zen/go/v1`, without user information, query parameters, or a fragment.
Each `credential` references a named entry below, independently of the provider name:

```toml
# ~/.agents/credentials.toml
[credentials.openai]
api_key = "replace-me"

[credentials.opencode]
api_key = "replace-me"
```

Replace placeholders before issuing requests, and never commit real credentials.
The initializer creates missing templates without overwriting existing files.
Configured providers must have resolvable credentials, non-empty model names, valid headers, and positive timeouts.
First-chunk and stream-idle timeouts default to 30 and 60 seconds respectively.
These are application streaming policies: the Rig agent type does not enforce them, so consumers must apply `first_chunk_timeout()` and `stream_idle_timeout()` around stream polling.
Other placeholders, including `${HOME}`, remain literal header text.

## API

- `AgentConfigPaths::standard()` selects `AGENTS_CONFIG` or the home-directory defaults.
- `AgentConfigPaths::new(config, credentials)` accepts explicit, absolute, different paths.
- `load_from_paths(paths)` reads existing files and validates providers and credentials.
- `load_or_initialize(paths)` additionally creates missing template files.
- `LoadedAgentsConfig::providers()` enumerates named providers, while `provider(name)` and `active_provider()` select one.
- `ResolvedProvider` exposes the validated URL, model, named credential, API key, expanded headers, additional parameters, and timeout durations for downstream adapters.
- `rig_agent_builder(session_id: Option<&str>)` returns a Rig-native `AgentBuilder` with connection, model, and request parameters applied, without building the agent.
- Callers add prompts and tools before calling `.build()`.

For a different library, implement `ProviderAdapter` with that library's configuration, request, or client type as `Output`:

```rust
use agents_config::{ProviderAdapter, ResolvedProvider};
use std::convert::Infallible;

struct ModelSelection;

impl ProviderAdapter for ModelSelection {
    type Output = String;
    type Error = Infallible;

    fn adapt(&self, provider: &ResolvedProvider, _session_id: Option<&str>) -> Result<String, Infallible> {
        Ok(provider.model().to_owned())
    }
}
```

Call `provider.adapt(&ModelSelection, session_id)` for this minimal adapter, or use the other resolved getters to construct a complete third-party client configuration.
No Rig feature is needed for custom adapters.

## Development

See [AGENTS.md](./AGENTS.md) for repository boundaries and validation commands.

## License

MIT. See [LICENSE](./LICENSE).

_This README was generated from the [share-artifact skill](https://raw.githubusercontent.com/totto2727-org/agent/refs/heads/main/plugins/totto2727-coding/skills/share-artifact/SKILL.md) and [README template](https://raw.githubusercontent.com/totto2727-org/agent/refs/heads/main/plugins/totto2727-coding/skills/share-artifact/readme/template.md)._
