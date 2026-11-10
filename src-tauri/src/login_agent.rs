use std::io;
use std::path::{Path, PathBuf};

pub const LOGIN_AGENT_LABEL: &str = "com.gestaobem.ram-bar";

/// LaunchAgent plist so RAM Bar starts in the Aqua session at login.
/// KeepAlive stays false: Sair must stick until the next login.
///
/// ```
/// let xml = ram_bar::login_agent::login_agent_plist("/Applications/RAM Bar.app/Contents/MacOS/ram-bar");
/// assert!(xml.contains("RunAtLoad"));
/// ```
pub fn login_agent_plist(program: &str) -> String {
    let exe = xml_escape(program);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LOGIN_AGENT_LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <false/>
  <key>LimitLoadToSessionType</key>
  <string>Aqua</string>
</dict>
</plist>
"#
    )
}

/// Only the binary inside a `.app/Contents/MacOS` bundle is a stable login target.
pub fn bundle_macos_executable(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    if macos.file_name()?.to_str()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()?.to_str()? != "Contents" {
        return None;
    }
    let bundle = contents.parent()?;
    if bundle.extension()?.to_str()? != "app" {
        return None;
    }
    Some(exe.to_path_buf())
}

pub fn write_login_agent(agents_dir: &Path, program: &str) -> io::Result<PathBuf> {
    std::fs::create_dir_all(agents_dir)?;
    let path = agents_dir.join(format!("{LOGIN_AGENT_LABEL}.plist"));
    std::fs::write(&path, login_agent_plist(program))?;
    Ok(path)
}

pub fn default_agents_dir() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is unset; cannot write ~/Library/LaunchAgents",
        )
    })?;
    Ok(PathBuf::from(home).join("Library/LaunchAgents"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use std::path::Path;

    fn fake_bundle_program() -> String {
        format!(
            "/Applications/App{}.app/Contents/MacOS/ram-bar",
            (1u32..9_000u32).fake::<u32>()
        )
    }

    #[test]
    fn plist_uses_identifier_run_at_load_and_aqua() {
        let program = fake_bundle_program();
        let plist = login_agent_plist(&program);
        assert!(plist.contains(&format!("<string>{LOGIN_AGENT_LABEL}</string>")));
        assert!(plist.contains("<key>RunAtLoad</key>\n  <true/>"));
        assert!(plist.contains("<key>KeepAlive</key>\n  <false/>"));
        assert!(plist.contains("<key>LimitLoadToSessionType</key>\n  <string>Aqua</string>"));
        assert!(plist.contains(&format!("<string>{program}</string>")));
    }

    #[test]
    fn plist_escapes_ampersand_and_angles() {
        let program = format!(
            "/tmp/{}&<>.app/Contents/MacOS/ram-bar",
            (1u32..9_000u32).fake::<u32>()
        );
        let plist = login_agent_plist(&program);
        assert!(!plist.contains(&program));
        assert!(plist.contains("&amp;&lt;&gt;.app"));
    }

    #[test]
    fn only_macos_bundle_executable_is_registered() {
        let program = fake_bundle_program();
        assert_eq!(
            bundle_macos_executable(Path::new(&program)).as_deref(),
            Some(Path::new(program.as_str()))
        );
        let debug = format!(
            "/Users/dev{}/target/debug/ram-bar",
            (1u32..9_000u32).fake::<u32>()
        );
        assert_eq!(bundle_macos_executable(Path::new(&debug)), None);
    }

    #[test]
    fn write_login_agent_creates_labeled_plist() {
        let dir = std::env::temp_dir().join(format!(
            "ram-bar-agent-{}",
            (1u32..1_000_000u32).fake::<u32>()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let program = fake_bundle_program();
        let path = write_login_agent(&dir, &program).unwrap();
        assert_eq!(path, dir.join(format!("{LOGIN_AGENT_LABEL}.plist")));
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body, login_agent_plist(&program));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
