use crate::utils::exec_cmd;
use clipboard::ClipboardProvider;
use reqwest;
use std::fs;
use std::path::PathBuf;

const GIT_IO_BASE_URL: &str = "https://git.io/";

pub fn create() {
    let os_info = os_info::get();

    let environment = Environment {
        os_type: os_info.os_type(),
        os_version: os_info.version().to_owned(),
        shell_info: get_shell_info(),
        terminal_info: get_terminal_info(),
        starship_config: get_starship_config(),
    };

    let link = get_github_issue_link();
    let env_info = format_env_info(crate_version!(), environment);
    let copy_success = clipboard::ClipboardProvider::new()
        .and_then(|mut ctx: clipboard::ClipboardContext| ctx.set_contents(env_info.to_string()))
        .map(|_| true)
        .unwrap_or(false);

    if open::that(&link).is_ok() {
        print!("Take a look at your browser. A GitHub issue has been created and your environment info has been copied to your clipboard.")
    } else {
        let link = reqwest::Client::new()
            .post(&format!("{}{}", GIT_IO_BASE_URL, "create"))
            .form(&[("url", &link)])
            .send()
            .and_then(|mut response| response.text())
            .map(|slug| format!("{}{}", GIT_IO_BASE_URL, slug))
            .unwrap_or(link);

        println!(
            "Your environment info has been copied to your clipboard. Click this link to create a GitHub issue:\n\n  {}",
            link
        );
    }

    if !copy_success {
        println!(
            "\n\nYour clipboard was unavailable, so here's the summary of your environment:\n\n{}",
            env_info
        );
    }
}

const UNKNOWN_SHELL: &str = "<unknown shell>";
const UNKNOWN_TERMINAL: &str = "<unknown terminal>";
const UNKNOWN_VERSION: &str = "<unknown version>";
const UNKNOWN_CONFIG: &str = "<unknown config>";

struct Environment {
    os_type: os_info::Type,
    os_version: os_info::Version,
    shell_info: ShellInfo,
    terminal_info: TerminalInfo,
    starship_config: String,
}

fn get_github_issue_link() -> String {
    let body = urlencoding::encode(&format!(
        "#### Current Behavior
<!-- A clear and concise description of the behavior. -->

#### Expected Behavior
<!-- A clear and concise description of what you expected to happen. -->

#### Additional context/Screenshots
<!-- Add any other context about the problem here. If applicable, add screenshots to help explain. -->

#### Possible Solution
<!--- Only if you have suggestions on a fix for the bug -->

#### Environment
<!-- Your environment information has been copied to the clipboard automatically. Paste it here. -->
",
    ))
        .replace("%20", "+");

    format!(
        "https://github.com/starship/starship/issues/new?body={}",
        body
    )
}

fn format_env_info(starship_version: &str, environment: Environment) -> String {
    format!(
        "- Starship version: {starship_version}
- {shell_name} version: {shell_version}
- Operating system: {os_name} {os_version}
- Terminal emulator: {terminal_name} {terminal_version}

#### Relevant Shell Configuration

```bash
{shell_config}
```

#### Starship Configuration

```toml
{starship_config}
```",
        starship_version = starship_version,
        shell_name = environment.shell_info.name,
        shell_version = environment.shell_info.version,
        os_name = environment.os_type,
        os_version = environment.os_version,
        terminal_name = environment.terminal_info.name,
        terminal_version = environment.terminal_info.version,
        shell_config = environment.shell_info.config,
        starship_config = environment.starship_config,
    )
}

#[derive(Debug)]
struct ShellInfo {
    name: String,
    version: String,
    config: String,
}

fn get_shell_info() -> ShellInfo {
    let shell = std::env::var("STARSHIP_SHELL");
    if shell.is_err() {
        return ShellInfo {
            name: UNKNOWN_SHELL.to_string(),
            version: UNKNOWN_VERSION.to_string(),
            config: UNKNOWN_CONFIG.to_string(),
        };
    }

    let shell = shell.unwrap();

    let version = exec_cmd(&shell, &["--version"])
        .map(|output| output.stdout.trim().to_string())
        .unwrap_or_else(|| UNKNOWN_VERSION.to_string());

    let config = get_config_path(&shell)
        .and_then(|config_path| fs::read_to_string(config_path).ok())
        .map(|config| config.trim().to_string())
        .unwrap_or_else(|| UNKNOWN_CONFIG.to_string());

    ShellInfo {
        name: shell,
        version,
        config,
    }
}

#[derive(Debug)]
struct TerminalInfo {
    name: String,
    version: String,
}

fn get_terminal_info() -> TerminalInfo {
    let terminal = std::env::var("TERM_PROGRAM")
        .or_else(|_| std::env::var("LC_TERMINAL"))
        .unwrap_or_else(|_| UNKNOWN_TERMINAL.to_string());

    let version = std::env::var("TERM_PROGRAM_VERSION")
        .or_else(|_| std::env::var("LC_TERMINAL_VERSION"))
        .unwrap_or_else(|_| UNKNOWN_VERSION.to_string());

    TerminalInfo {
        name: terminal,
        version,
    }
}

fn get_config_path(shell: &str) -> Option<PathBuf> {
    dirs::home_dir().and_then(|home_dir| {
        match shell {
            "bash" => Some(".bashrc"),
            "fish" => Some(".config/fish/config.fish"),
            "ion" => Some("~/.config/ion/initrc"),
            "powershell" => {
                if cfg!(windows) {
                    Some("Documents/PowerShell/Microsoft.PowerShell_profile.ps1")
                } else {
                    Some(".config/powershell/Microsoft.PowerShell_profile.ps1")
                }
            }
            "zsh" => Some(".zshrc"),
            _ => None,
        }
        .map(|path| home_dir.join(path))
    })
}

fn get_starship_config() -> String {
    dirs::home_dir()
        .and_then(|home_dir| fs::read_to_string(home_dir.join(".config/starship.toml")).ok())
        .unwrap_or_else(|| UNKNOWN_CONFIG.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_info;
    use std::env;

    #[test]
    fn test_format_env_info() {
        let starship_version = "0.1.2";
        let environment = Environment {
            os_type: os_info::Type::Linux,
            os_version: os_info::Version::semantic(1, 2, 3, Some("test".to_string())),
            shell_info: ShellInfo {
                name: "test_shell".to_string(),
                version: "2.3.4".to_string(),
                config: "No config".to_string(),
            },
            terminal_info: TerminalInfo {
                name: "test_terminal".to_string(),
                version: "5.6.7".to_string(),
            },
            starship_config: "No Starship config".to_string(),
        };

        let env_info = format_env_info(starship_version, environment);

        assert!(env_info.contains(starship_version));
        assert!(env_info.contains("Linux"));
        assert!(env_info.contains("1.2.3"));
        assert!(env_info.contains("test_shell"));
        assert!(env_info.contains("2.3.4"));
        assert!(env_info.contains("No config"));
        assert!(env_info.contains("No Starship config"));
    }

    #[test]
    fn test_get_shell_info() {
        env::remove_var("STARSHIP_SHELL");
        let unknown_shell = get_shell_info();
        assert_eq!(UNKNOWN_SHELL, &unknown_shell.name);

        env::set_var("STARSHIP_SHELL", "fish");

        let fish_shell = get_shell_info();
        assert_eq!("fish", &fish_shell.name);
    }

    #[test]
    #[cfg(not(windows))]
    fn test_get_config_path() {
        env::set_var("HOME", "/test/home");

        let config_path = get_config_path("bash");
        assert_eq!("/test/home/.bashrc", config_path.unwrap().to_str().unwrap());
    }
}
