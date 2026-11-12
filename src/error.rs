//! Typed errors for certificate verification.

/// Why a certificate failed verification.
#[derive(Debug)]
#[non_exhaustive]
pub enum VerifyError {
    /// The record format is newer than this crate understands.
    Version {
        /// Version found in the record.
        found: u32,
        /// Highest version this crate verifies.
        supported: u32,
    },
    /// The source hash no longer matches the current source.
    StaleSource {
        /// The theorem whose badge is stale.
        theorem: String,
        /// The hash recorded at verification time.
        recorded: String,
        /// The hash of the current source.
        current: String,
    },
    /// The toolchain no longer matches the one that ran the kernel.
    StaleToolchain {
        /// The theorem whose badge is stale.
        theorem: String,
        /// The toolchain recorded at verification time.
        recorded: String,
        /// The current toolchain identifier.
        current: String,
    },
    /// The record does not claim kernel verification.
    NotVerified {
        /// The theorem the record names.
        theorem: String,
    },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Version { found, supported } => {
                write!(f, "certificate version {found} unsupported; this crate verifies up to {supported}")
            }
            Self::StaleSource {
                theorem,
                recorded,
                current,
            } => write!(
                f,
                "stale proof badge for `{theorem}`: recorded source hash {recorded}, current source hashes to {current}"
            ),
            Self::StaleToolchain {
                theorem,
                recorded,
                current,
            } => write!(
                f,
                "stale proof badge for `{theorem}`: recorded toolchain {recorded}, current toolchain {current}"
            ),
            Self::NotVerified { theorem } => {
                write!(f, "certificate for `{theorem}` does not record kernel verification")
            }
        }
    }
}

impl std::error::Error for VerifyError {}

/// Why a badge failed structural validation (the client-side checker's
/// error type; REQ-P4).
#[derive(Debug)]
#[non_exhaustive]
pub enum BadgeError {
    /// The badge is not valid JSON or does not match the record shape.
    Malformed(String),
    /// The record format is newer than this checker understands.
    Version {
        /// Version found in the badge.
        found: u32,
        /// Highest version this checker validates.
        supported: u32,
    },
    /// The source hash is not 64 lowercase hex characters.
    BadSourceHash {
        /// The hash string as found.
        found: String,
    },
    /// The badge names no toolchain, so provenance is unverifiable.
    EmptyToolchain,
    /// The badge does not record kernel verification.
    NotVerified {
        /// The theorem the badge names.
        theorem: String,
    },
}

impl std::fmt::Display for BadgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed(e) => write!(f, "malformed badge: {e}"),
            Self::Version { found, supported } => {
                write!(
                    f,
                    "badge version {found} unsupported; this checker validates up to {supported}"
                )
            }
            Self::BadSourceHash { found } => {
                write!(f, "badge source hash is not 64 lowercase hex: {found}")
            }
            Self::EmptyToolchain => write!(f, "badge records no toolchain"),
            Self::NotVerified { theorem } => {
                write!(
                    f,
                    "badge for `{theorem}` does not record kernel verification"
                )
            }
        }
    }
}

impl std::error::Error for BadgeError {}
