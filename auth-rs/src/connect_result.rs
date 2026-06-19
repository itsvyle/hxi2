use connectrpc::{ConnectError, ErrorCode};

use crate::app_config;

pub struct ObfuscatedResult<T>(Result<T, anyhow::Error>);

#[allow(dead_code)]
pub trait ToConnectError<T> {
    fn obfuscate(self) -> ObfuscatedResult<T>
    where
        Self: Sized;
    fn to_connect_err(self) -> Result<T, ConnectError>;
    fn to_connect_internal(self) -> Result<T, ConnectError>;
    fn to_connect_permission_denied(self) -> Result<T, ConnectError>;
    fn to_connect_unimplemented(self) -> Result<T, ConnectError>;
    fn to_connect_invalid_argument(self) -> Result<T, ConnectError>;
    fn to_connect_not_found(self) -> Result<T, ConnectError>;
    fn to_connect_failed_precondition(self) -> Result<T, ConnectError>;
    fn to_connect_resource_exhausted(self) -> Result<T, ConnectError>;
    fn to_connect_unavailable(self) -> Result<T, ConnectError>;
    fn to_connect_data_loss(self) -> Result<T, ConnectError>;
    fn to_connect_unauthenticated(self) -> Result<T, ConnectError>;
}

// in dev mode, give everything; in prod mode, obfuscate the error chain and only give the outer context if it exists...
fn resolve_secure_msg(e: anyhow::Error, title: &str) -> String {
    if app_config::AppConfiguration::INSTANCE().is_development() {
        format!("{}: {:?}", title, e)
    } else {
        let mut chain = e.chain();
        let outer_context = chain.next().map(|c| c.to_string());
        let has_root_cause = chain.next().is_some();

        if has_root_cause {
            format!(
                "{}: {} (error details hidden)",
                title,
                outer_context.unwrap_or_else(|| "unknown error".to_string())
            )
        } else {
            title.to_string()
        }
    }
}

impl<T> ToConnectError<T> for Result<T, anyhow::Error> {
    fn obfuscate(self) -> ObfuscatedResult<T> {
        ObfuscatedResult(self)
    }

    fn to_connect_err(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::new(ErrorCode::Internal, e.to_string()))
    }

    fn to_connect_internal(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::internal(e.to_string()))
    }

    fn to_connect_permission_denied(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::permission_denied(e.to_string()))
    }

    fn to_connect_unimplemented(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::unimplemented(e.to_string()))
    }

    fn to_connect_invalid_argument(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::invalid_argument(e.to_string()))
    }

    fn to_connect_not_found(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::not_found(e.to_string()))
    }

    fn to_connect_failed_precondition(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::failed_precondition(e.to_string()))
    }

    fn to_connect_resource_exhausted(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::resource_exhausted(e.to_string()))
    }

    fn to_connect_unavailable(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::unavailable(e.to_string()))
    }

    fn to_connect_data_loss(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::data_loss(e.to_string()))
    }

    fn to_connect_unauthenticated(self) -> Result<T, ConnectError> {
        self.map_err(|e| ConnectError::unauthenticated(e.to_string()))
    }
}

impl<T> ToConnectError<T> for ObfuscatedResult<T> {
    fn obfuscate(self) -> ObfuscatedResult<T> {
        self
    }

    fn to_connect_err(self) -> Result<T, ConnectError> {
        self.to_connect_internal()
    }

    fn to_connect_internal(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "An internal server error occurred.");
            ConnectError::internal(msg)
        })
    }

    fn to_connect_not_found(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "The requested resource was not found.");
            ConnectError::not_found(msg)
        })
    }

    fn to_connect_permission_denied(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Permission denied.");
            ConnectError::permission_denied(msg)
        })
    }

    fn to_connect_unimplemented(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "This feature is not implemented.");
            ConnectError::unimplemented(msg)
        })
    }

    fn to_connect_invalid_argument(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Invalid argument provided.");
            ConnectError::invalid_argument(msg)
        })
    }

    fn to_connect_failed_precondition(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Failed precondition.");
            ConnectError::failed_precondition(msg)
        })
    }

    fn to_connect_resource_exhausted(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Resource exhausted.");
            ConnectError::resource_exhausted(msg)
        })
    }

    fn to_connect_unavailable(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Service unavailable.");
            ConnectError::unavailable(msg)
        })
    }

    fn to_connect_data_loss(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Data loss occurred.");
            ConnectError::data_loss(msg)
        })
    }

    fn to_connect_unauthenticated(self) -> Result<T, ConnectError> {
        self.0.map_err(|e| {
            let msg = resolve_secure_msg(e, "Unauthenticated.");
            ConnectError::unauthenticated(msg)
        })
    }
}
