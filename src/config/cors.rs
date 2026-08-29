//! CORS configuration parsing and middleware builder.

use actix_cors::Cors;
use serde::Deserialize;

/// Represents either a single string value or multiple string values.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum OneOrMany {
    /// Single value (e.g. `"*"` or `"https://example.com"`).
    One(String),
    /// Multiple values (e.g. `["https://a.com", "https://b.com"]`).
    Many(Vec<String>),
}

/// Structured settings for CORS middleware.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CorsSettings {
    /// Allowed origins (or `*`).
    pub origins: Option<OneOrMany>,
    /// Allowed HTTP methods (or `*`).
    pub methods: Option<OneOrMany>,
    /// Allowed HTTP headers (or `*`).
    pub headers: Option<OneOrMany>,
    /// Exposed HTTP response headers (or `*`).
    #[serde(rename = "expose-headers")]
    pub expose_headers: Option<OneOrMany>,
    /// Whether to allow user credentials (cookies, auth headers).
    pub credentials: bool,
    /// Whether to send wildcard origin in response.
    #[serde(rename = "send-wildcard")]
    pub send_wildcard: bool,
    /// Maximum cache age for preflight requests in seconds.
    #[serde(rename = "max-age")]
    pub max_age: Option<usize>,
    /// Whether to block requests on origin mismatch.
    #[serde(rename = "block-on-origin-mismatch")]
    pub block_on_origin_mismatch: bool,
}

impl CorsSettings {
    /// Loads CORS configuration from environment variables.
    ///
    /// If `ENABLE_CORS` is explicitly set to `false`, or if no CORS environment variables
    /// (`ENABLE_CORS`, `CORS_ORIGINS`, etc.) are found in the environment, this function
    /// returns `None`, disabling CORS by default.
    pub fn from_env() -> Option<Self> {
        let enable_cors = std::env::var("ENABLE_CORS")
            .or_else(|_| std::env::var("CORS_ENABLED"))
            .ok()
            .and_then(|s| s.parse::<bool>().ok());

        let origins = std::env::var("CORS_ORIGINS").ok().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "*" {
                Some(OneOrMany::One(trimmed.to_string()))
            } else {
                Some(OneOrMany::Many(
                    trimmed.split(',').map(|s| s.trim().to_string()).collect(),
                ))
            }
        });

        let methods = std::env::var("CORS_METHODS").ok().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "*" {
                Some(OneOrMany::One(trimmed.to_string()))
            } else {
                Some(OneOrMany::Many(
                    trimmed.split(',').map(|s| s.trim().to_string()).collect(),
                ))
            }
        });

        let headers = std::env::var("CORS_HEADERS").ok().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "*" {
                Some(OneOrMany::One(trimmed.to_string()))
            } else {
                Some(OneOrMany::Many(
                    trimmed.split(',').map(|s| s.trim().to_string()).collect(),
                ))
            }
        });

        let expose_headers = std::env::var("CORS_EXPOSE_HEADERS").ok().and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed == "*" {
                Some(OneOrMany::One(trimmed.to_string()))
            } else {
                Some(OneOrMany::Many(
                    trimmed.split(',').map(|s| s.trim().to_string()).collect(),
                ))
            }
        });

        let credentials = std::env::var("CORS_CREDENTIALS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let send_wildcard = std::env::var("CORS_SEND_WILDCARD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let max_age = std::env::var("CORS_MAX_AGE")
            .ok()
            .and_then(|s| s.parse().ok());

        let block_on_origin_mismatch = std::env::var("CORS_BLOCK_ON_ORIGIN_MISMATCH")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        // Check if any CORS environment variables were defined
        let has_any_cors_config = origins.is_some()
            || methods.is_some()
            || headers.is_some()
            || expose_headers.is_some()
            || max_age.is_some()
            || std::env::var("CORS_CREDENTIALS").is_ok()
            || std::env::var("CORS_SEND_WILDCARD").is_ok()
            || std::env::var("CORS_BLOCK_ON_ORIGIN_MISMATCH").is_ok();

        let is_enabled = match enable_cors {
            Some(true) => true,
            Some(false) => false,
            None => has_any_cors_config,
        };

        if !is_enabled {
            return None;
        }

        Some(CorsSettings {
            origins,
            methods,
            headers,
            expose_headers,
            credentials,
            send_wildcard,
            max_age,
            block_on_origin_mismatch,
        })
    }
}

/// Builds an [`actix_cors::Cors`] middleware instance from [`CorsSettings`].
pub fn cors_from_settings(settings: &CorsSettings) -> Cors {
    let mut cors = Cors::default();

    match settings.origins.as_ref() {
        None => {}
        Some(OneOrMany::One(origin)) if origin == "*" => {
            cors = cors.allow_any_origin();
        }
        Some(OneOrMany::One(origin)) => {
            cors = cors.allowed_origin(origin);
        }
        Some(OneOrMany::Many(origins)) => {
            for origin in origins {
                cors = cors.allowed_origin(origin);
            }
        }
    }

    match settings.methods.as_ref() {
        None => {}
        Some(OneOrMany::One(method)) if method == "*" => {
            cors = cors.allow_any_method();
        }
        Some(OneOrMany::One(method)) => {
            cors = cors.allowed_methods([method.as_str()]);
        }
        Some(OneOrMany::Many(methods)) => {
            cors = cors.allowed_methods(methods.iter().map(String::as_str));
        }
    }

    match settings.headers.as_ref() {
        None => {}
        Some(OneOrMany::One(header)) if header == "*" => {
            cors = cors.allow_any_header();
        }
        Some(OneOrMany::One(header)) => {
            cors = cors.allowed_header(header.as_str());
        }
        Some(OneOrMany::Many(headers)) => {
            cors = cors.allowed_headers(headers.iter().map(String::as_str));
        }
    }

    match settings.expose_headers.as_ref() {
        None => {}
        Some(OneOrMany::One(header)) if header == "*" => {
            cors = cors.expose_any_header();
        }
        Some(OneOrMany::One(header)) => {
            cors = cors.expose_headers([header.as_str()]);
        }
        Some(OneOrMany::Many(headers)) => {
            cors = cors.expose_headers(headers.iter().map(String::as_str));
        }
    }

    if settings.credentials {
        cors = cors.supports_credentials();
    }

    if settings.send_wildcard {
        cors = cors.send_wildcard();
    }

    if let Some(max_age) = settings.max_age {
        cors = cors.max_age(max_age);
    }

    cors.block_on_origin_mismatch(settings.block_on_origin_mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_by_default_when_no_env() {
        // Ensure no CORS env vars
        unsafe {
            std::env::remove_var("ENABLE_CORS");
            std::env::remove_var("CORS_ENABLED");
            std::env::remove_var("CORS_ORIGINS");
            std::env::remove_var("CORS_METHODS");
            std::env::remove_var("CORS_HEADERS");
            std::env::remove_var("CORS_EXPOSE_HEADERS");
            std::env::remove_var("CORS_CREDENTIALS");
            std::env::remove_var("CORS_SEND_WILDCARD");
            std::env::remove_var("CORS_MAX_AGE");
            std::env::remove_var("CORS_BLOCK_ON_ORIGIN_MISMATCH");
        }

        assert!(CorsSettings::from_env().is_none());
    }

    #[test]
    fn test_enabled_when_origins_set() {
        unsafe {
            std::env::set_var(
                "CORS_ORIGINS",
                "http://localhost:3000,https://app.example.com",
            );
        }

        let settings = CorsSettings::from_env().expect("Should be enabled when origins are set");
        assert_eq!(
            settings.origins,
            Some(OneOrMany::Many(vec![
                "http://localhost:3000".to_string(),
                "https://app.example.com".to_string()
            ]))
        );

        unsafe {
            std::env::remove_var("CORS_ORIGINS");
        }
    }

    #[test]
    fn test_explicitly_disabled() {
        unsafe {
            std::env::set_var("ENABLE_CORS", "false");
            std::env::set_var("CORS_ORIGINS", "*");
        }

        assert!(CorsSettings::from_env().is_none());

        unsafe {
            std::env::remove_var("ENABLE_CORS");
            std::env::remove_var("CORS_ORIGINS");
        }
    }
}
