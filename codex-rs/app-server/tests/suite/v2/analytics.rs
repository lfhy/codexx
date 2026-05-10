use anyhow::Result;
use codex_core::config::ConfigBuilder;
use tempfile::TempDir;

const SERVICE_VERSION: &str = "0.0.0-test";

#[tokio::test]
async fn app_server_default_analytics_disabled_without_flag() -> Result<()> {
    let codex_home = TempDir::new()?;
    let mut config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await?;
    config.analytics_enabled = None;

    let provider = codex_core::otel_init::build_provider(
        &config,
        SERVICE_VERSION,
        Some("codex-app-server"),
        /*default_analytics_enabled*/ false,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    assert!(provider.is_none());
    Ok(())
}

#[tokio::test]
async fn app_server_default_analytics_enabled_with_flag() -> Result<()> {
    let codex_home = TempDir::new()?;
    let mut config = ConfigBuilder::default()
        .codex_home(codex_home.path().to_path_buf())
        .build()
        .await?;
    config.analytics_enabled = None;

    let provider = codex_core::otel_init::build_provider(
        &config,
        SERVICE_VERSION,
        Some("codex-app-server"),
        /*default_analytics_enabled*/ true,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    assert!(provider.is_none());
    Ok(())
}
