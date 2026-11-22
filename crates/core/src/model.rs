use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsoleModel {
    Phat,
    Slim,
    Pro,
    Unknown,
}

/// Where a firmware string came from. Never "verified" in v0.1: no source can prove compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FirmwareSource {
    Unknown,
    /// Coarse hint from the discovery reply `system-version` header.
    Reported,
    UserEntered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Firmware {
    pub value: Option<String>,
    pub source: FirmwareSource,
}

impl Default for Firmware {
    fn default() -> Self {
        Self {
            value: None,
            source: FirmwareSource::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsoleProfile {
    pub id: i64,
    pub name: String,
    pub model: ConsoleModel,
    /// IPv4/IPv6 literal; `None` for the simulator profile.
    pub address: Option<String>,
    pub firmware: Firmware,
    pub simulated: bool,
    pub last_connected: Option<i64>,
}

/// Validates a user-typed console name/address before it is stored.
pub fn validate_profile_input(name: &str, address: Option<&str>) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 48 {
        return Err("Name must be 1–48 characters.".into());
    }
    if name.chars().any(char::is_control) {
        return Err("Name contains control characters.".into());
    }
    if let Some(a) = address {
        a.parse::<std::net::IpAddr>()
            .map_err(|_| "Address must be an IP address such as 192.168.1.50.".to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_input() {
        assert!(validate_profile_input("", None).is_err());
        assert!(validate_profile_input("ok", Some("not-an-ip")).is_err());
        assert!(validate_profile_input("ok", Some("192.168.1.2; rm")).is_err());
        assert!(validate_profile_input("bad\u{7}", None).is_err());
        assert!(validate_profile_input("My PS4", Some("192.168.1.50")).is_ok());
        assert!(validate_profile_input("My PS4", Some("fe80::1")).is_ok());
    }
}
