use crate::api::Platforms;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
    Other,
}

pub fn detect_os() -> Os {
    let Some(navitator) = web_sys::window().map(|w| w.navigator()) else {
        return Os::Other;
    };
    let hinted = js_sys::Reflect::get(navitator.as_ref(), &"userAgentData".into())
        .ok()
        .and_then(|data| js_sys::Reflect::get(&data, &"platform".into()).ok())
        .and_then(|p| p.as_string())
        .unwrap_or_default();

    let raw = format!("{hinted} {}", navitator.user_agent().unwrap_or_default()).to_lowercase();

    if raw.contains("android") {
        Os::Other
    } else if raw.contains("win") {
        Os::Windows
    } else if raw.contains("mac") || raw.contains("iphone") || raw.contains("ipad") {
        Os::MacOs
    } else if raw.contains("linux") || raw.contains("x11") {
        Os::Linux
    } else {
        Os::Other
    }
}

fn platform_os(key: &str) -> Os {
    if key.starts_with("windows") {
        Os::Windows
    } else if key.starts_with("darwin") {
        Os::MacOs
    } else if key.starts_with("linux") {
        Os::Linux
    } else {
        Os::Other
    }
}

pub fn platform_label(key: &str) -> String {
    match key {
        "windows-x86_64" => "Windows (x64)",
        "darwin-x86_64" => "macOS (Intel)",
        "darwin-aarch64" => "macOS (Apple Silicon)",
        "linux-x86_64" => "Linux (x64)",
        "linux-x86_64-rpm" => "Linux — Fedora / RHEL (.rpm)",
        "linux-x86_64-pacman" => "Linux — Arch (.pkg.tar.zst)",
        "linux-x86_64-deb" => "Linux — Debian / Ubuntu (.deb)",
        other => other,
    }
    .to_string()
}

pub fn pick_platform_for(os: Os, platforms: &Platforms) -> Option<String> {
    let matches: Vec<&String> = platforms
        .keys()
        .filter(|k| platform_os(k) == os)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return None;
    }
    if os != Os::Linux {
        return matches.first().map(|k| (*k).clone());
    }

    let mut preference = vec!["linux-x86_64-rpm", "linux-x86_64-deb", "linux-x86_64"];
    let ua = web_sys::window()
        .map(|w| w.navigator())
        .and_then(|n| n.user_agent().ok())
        .unwrap_or_default()
        .to_lowercase();
    if ["arch", "manjaro", "endeavour"]
        .iter()
        .any(|d| ua.contains(d))
    {
        preference.insert(0, "linux-x86_64-pacman");
    }

    preference
        .into_iter()
        .find(|k| matches.iter().any(|m| m.as_str() == *k))
        .map(str::to_string)
        .or_else(|| matches.first().map(|k| (*k).clone()))
}
