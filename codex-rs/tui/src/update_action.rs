use crate::legacy_core::config::UpdateConfig;
use codex_install_context::InstallContext;
use codex_install_context::StandalonePlatform;

/// Update action the CLI should perform after the TUI exits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Update via `npm install -g @openai/codex@latest`.
    NpmGlobalLatest,
    /// Update via `bun install -g @openai/codex@latest`.
    BunGlobalLatest,
    /// Update via `brew upgrade codex`.
    BrewUpgrade,
    /// Update via `curl -fsSL https://chatgpt.com/codex/install.sh | sh`.
    StandaloneUnix,
    /// Update via `irm https://chatgpt.com/codex/install.ps1|iex`.
    StandaloneWindows,
}

impl UpdateAction {
    pub(crate) fn from_install_context(context: &InstallContext) -> Option<Self> {
        match context {
            InstallContext::Npm => Some(UpdateAction::NpmGlobalLatest),
            InstallContext::Bun => Some(UpdateAction::BunGlobalLatest),
            InstallContext::Brew => Some(UpdateAction::BrewUpgrade),
            InstallContext::Standalone { platform, .. } => Some(match platform {
                StandalonePlatform::Unix => UpdateAction::StandaloneUnix,
                StandalonePlatform::Windows => UpdateAction::StandaloneWindows,
            }),
            InstallContext::Other => None,
        }
    }

    /// Returns the list of command-line arguments for invoking the update.
    pub fn command_args(self) -> (&'static str, &'static [&'static str]) {
        match self {
            UpdateAction::NpmGlobalLatest => ("npm", &["install", "-g", "@openai/codex"]),
            UpdateAction::BunGlobalLatest => ("bun", &["install", "-g", "@openai/codex"]),
            UpdateAction::BrewUpgrade => ("brew", &["upgrade", "--cask", "codex"]),
            UpdateAction::StandaloneUnix => (
                "sh",
                &["-c", "curl -fsSL https://chatgpt.com/codex/install.sh | sh"],
            ),
            UpdateAction::StandaloneWindows => (
                "powershell",
                &["-c", "irm https://chatgpt.com/codex/install.ps1|iex"],
            ),
        }
    }

    /// Returns string representation of the command-line arguments for invoking the update.
    pub fn command_str(self) -> String {
        let (command, args) = self.command_args();
        shlex::try_join(std::iter::once(command).chain(args.iter().copied()))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }

    /// Returns command arguments using the configurable update URLs.
    pub fn command_args_with_config(self, update_config: &UpdateConfig) -> (String, Vec<String>) {
        match self {
            UpdateAction::NpmGlobalLatest => (
                "npm".to_string(),
                vec![
                    "install".to_string(),
                    "-g".to_string(),
                    "@openai/codex".to_string(),
                ],
            ),
            UpdateAction::BunGlobalLatest => (
                "bun".to_string(),
                vec![
                    "install".to_string(),
                    "-g".to_string(),
                    "@openai/codex".to_string(),
                ],
            ),
            UpdateAction::BrewUpgrade => (
                "brew".to_string(),
                vec![
                    "upgrade".to_string(),
                    "--cask".to_string(),
                    "codex".to_string(),
                ],
            ),
            UpdateAction::StandaloneUnix => (
                "sh".to_string(),
                vec![
                    "-c".to_string(),
                    format!(
                        "curl -fsSL {} | sh",
                        update_config.standalone_unix_installer_url
                    ),
                ],
            ),
            UpdateAction::StandaloneWindows => (
                "powershell".to_string(),
                vec![
                    "-c".to_string(),
                    format!("irm {}|iex", update_config.standalone_windows_installer_url),
                ],
            ),
        }
    }

    /// Returns the command string using the configurable update URLs.
    pub fn command_str_with_config(self, update_config: &UpdateConfig) -> String {
        let (command, args) = self.command_args_with_config(update_config);
        shlex::try_join(std::iter::once(command.as_str()).chain(args.iter().map(String::as_str)))
            .unwrap_or_else(|_| format!("{command} {}", args.join(" ")))
    }
}

pub fn get_update_action() -> Option<UpdateAction> {
    UpdateAction::from_install_context(InstallContext::current())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legacy_core::config::UpdateConfig;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn maps_install_context_to_update_action() {
        let native_release_dir = PathBuf::from("/tmp/native-release");

        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Other),
            None
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Npm),
            Some(UpdateAction::NpmGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Bun),
            Some(UpdateAction::BunGlobalLatest)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Brew),
            Some(UpdateAction::BrewUpgrade)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Standalone {
                platform: StandalonePlatform::Unix,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            }),
            Some(UpdateAction::StandaloneUnix)
        );
        assert_eq!(
            UpdateAction::from_install_context(&InstallContext::Standalone {
                platform: StandalonePlatform::Windows,
                release_dir: native_release_dir.clone(),
                resources_dir: Some(native_release_dir.join("codex-resources")),
            }),
            Some(UpdateAction::StandaloneWindows)
        );
    }

    #[test]
    fn standalone_update_commands_rerun_latest_installer() {
        assert_eq!(
            UpdateAction::StandaloneUnix.command_args(),
            (
                "sh",
                &["-c", "curl -fsSL https://chatgpt.com/codex/install.sh | sh"][..],
            )
        );
        assert_eq!(
            UpdateAction::StandaloneWindows.command_args(),
            (
                "powershell",
                &["-c", "irm https://chatgpt.com/codex/install.ps1|iex"][..],
            )
        );
    }

    #[test]
    fn standalone_update_commands_use_configured_urls() {
        let update_config = UpdateConfig {
            enabled: false,
            latest_release_api_url: "https://updates.internal/latest".to_string(),
            release_notes_url: "https://updates.internal/releases".to_string(),
            install_options_url: "https://updates.internal/install".to_string(),
            homebrew_cask_api_url: "https://updates.internal/brew.json".to_string(),
            npm_package_url: "https://updates.internal/npm".to_string(),
            standalone_unix_installer_url: "https://updates.internal/install.sh".to_string(),
            standalone_windows_installer_url: "https://updates.internal/install.ps1".to_string(),
        };

        assert_eq!(
            UpdateAction::StandaloneUnix.command_str_with_config(&update_config),
            "sh -c 'curl -fsSL https://updates.internal/install.sh | sh'"
        );
        assert_eq!(
            UpdateAction::StandaloneWindows.command_str_with_config(&update_config),
            "powershell -c 'irm https://updates.internal/install.ps1|iex'"
        );
    }
}
