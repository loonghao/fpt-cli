use fpt_domain::{AuthMode, ConnectionOverrides, ConnectionSettings, Credentials};
use std::env;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvSnapshot {
    key: String,
    previous: Option<String>,
}

struct EnvGuard {
    snapshots: Vec<EnvSnapshot>,
}

impl EnvGuard {
    fn set(vars: &[(&str, Option<&str>)]) -> Self {
        let mut snapshots = Vec::with_capacity(vars.len());
        for (key, value) in vars {
            snapshots.push(EnvSnapshot {
                key: (*key).to_string(),
                previous: env::var(key).ok(),
            });
            unsafe {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }
        }
        Self { snapshots }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for snapshot in &self.snapshots {
            unsafe {
                match &snapshot.previous {
                    Some(value) => env::set_var(&snapshot.key, value),
                    None => env::remove_var(&snapshot.key),
                }
            }
        }
    }
}

#[test]
fn resolves_script_auth_with_explicit_mode() {
    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::Script),
        script_name: Some("bot".to_string()),
        script_key: Some("secret".to_string()),
        ..Default::default()
    })
    .expect("script auth resolves");

    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "bot");
            assert_eq!(script_key, "secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn infers_user_password_auth_from_username_and_password() {
    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        username: Some("artist@example.com".to_string()),
        password: Some("secret".to_string()),
        ..Default::default()
    })
    .expect("user password auth resolves");

    assert_eq!(settings.auth_mode(), AuthMode::UserPassword);
    match settings.credentials {
        Credentials::UserPassword {
            username,
            password,
            auth_token,
        } => {
            assert_eq!(username, "artist@example.com");
            assert_eq!(password, "secret");
            assert!(auth_token.is_none());
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn resolves_session_token_auth_with_explicit_mode() {
    let settings = ConnectionSettings::resolve(ConnectionOverrides {
        site: Some("https://example.shotgrid.autodesk.com".to_string()),
        auth_mode: Some(AuthMode::SessionToken),
        session_token: Some("session-123".to_string()),
        ..Default::default()
    })
    .expect("session token auth resolves");

    assert_eq!(settings.auth_mode(), AuthMode::SessionToken);
    match settings.credentials {
        Credentials::SessionToken { session_token } => {
            assert_eq!(session_token, "session-123");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn resolves_script_auth_from_sg_env_fallback() {
    let _lock = env_lock().lock().expect("env lock");
    let _guard = EnvGuard::set(&[
        ("SG_SITE", Some("https://sg-env.shotgrid.autodesk.com")),
        ("SG_SCRIPT_NAME", Some("sg-bot")),
        ("SG_SCRIPT_KEY", Some("sg-secret")),
        ("FPT_SITE", None),
        ("FPT_SCRIPT_NAME", None),
        ("FPT_SCRIPT_KEY", None),
        ("FPT_AUTH_MODE", None),
        ("SG_AUTH_MODE", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_SESSION_TOKEN", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides::default())
        .expect("script auth resolves from SG env");

    assert_eq!(settings.site, "https://sg-env.shotgrid.autodesk.com");
    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "sg-bot");
            assert_eq!(script_key, "sg-secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}

#[test]
fn prefers_fpt_env_over_sg_env_when_both_present() {
    let _lock = env_lock().lock().expect("env lock");
    let _guard = EnvGuard::set(&[
        ("FPT_SITE", Some("https://fpt-env.shotgrid.autodesk.com")),
        ("FPT_SCRIPT_NAME", Some("fpt-bot")),
        ("FPT_SCRIPT_KEY", Some("fpt-secret")),
        ("SG_SITE", Some("https://sg-env.shotgrid.autodesk.com")),
        ("SG_SCRIPT_NAME", Some("sg-bot")),
        ("SG_SCRIPT_KEY", Some("sg-secret")),
        ("FPT_AUTH_MODE", None),
        ("SG_AUTH_MODE", None),
        ("FPT_USERNAME", None),
        ("FPT_PASSWORD", None),
        ("FPT_AUTH_TOKEN", None),
        ("FPT_SESSION_TOKEN", None),
        ("SG_USERNAME", None),
        ("SG_PASSWORD", None),
        ("SG_AUTH_TOKEN", None),
        ("SG_SESSION_TOKEN", None),
    ]);

    let settings = ConnectionSettings::resolve(ConnectionOverrides::default())
        .expect("script auth resolves from FPT env");

    assert_eq!(settings.site, "https://fpt-env.shotgrid.autodesk.com");
    assert_eq!(settings.auth_mode(), AuthMode::Script);
    match settings.credentials {
        Credentials::Script {
            script_name,
            script_key,
        } => {
            assert_eq!(script_name, "fpt-bot");
            assert_eq!(script_key, "fpt-secret");
        }
        other => panic!("unexpected credentials: {other:?}"),
    }
}
