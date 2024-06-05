use std::fmt::{Display, Formatter};

use strum_macros::{AsRefStr, EnumIter, EnumString, FromRepr};

/// XXXReadonly：RegionReadonly
/// XXXAlreadyExists XXXExists：TableAlreadyExists
/// XXXNotFound：UserNotFound、TableNotFound
/// XXXNotReady：RegionNotReady
/// XXXUnavailable：StorageUnavailable
/// XXXMismatch：UserPasswordMismatch
/// XXXDenied：AccessDenied、PermissionDenied
/// InvalidXXX: InvalidArguments、InvalidSyntax
/// Unsupported UnsupportedXXX：UnsupportedPasswordType
/// IllegalXXX：IllegalArguments
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, EnumString, EnumIter, FromRepr, AsRefStr)]
pub enum StatusCode {
    // ====== common status code ========
    /// Success
    Success = 0,
    /// Unknown error.
    Unknown = 1000,
    /// Unsupported operation.
    Unsupported = 1001,
    /// Unexpected error, maybe there is a BUG.
    Unexpected = 1002,
    /// Internal server error.
    Internal = 1003,
    /// Invalid arguments.
    InvalidArguments = 1004,
    /// The task is cancelled.
    Cancelled = 1005,
    // ====== common status code ========

    /// Syntax error。
    InvalidSyntax = 2000,

    // ====== server related status code ========
    /// Runtime resources exhausted, like creating threads failed.
    RuntimeResourcesExhausted = 6000,

    /// Rate limit exceeded
    RateLimited = 6001,
    // ====== server related status code ========

    // ====== auth related status code ========
    /// User not found.
    UserNotFound = 7000,
    /// Unsupported password type.
    UnsupportedPasswordType = 7001,
    /// User and password does not match
    UserPasswordMismatch = 7002,
    /// Not found http authorization header
    AuthHeaderNotFound = 7003,
    /// Invalid http authorization header
    InvalidAuthHeader = 7004,
    /// access denied
    AccessDenied = 7005,
    /// permission denied
    PermissionDenied = 7006,
    // ====== auth related status code ========
}

impl StatusCode {
    pub fn is_success(code: u32) -> bool {
        Self::Success as u32 == code
    }

    pub fn from_u32(code: u32) -> Option<Self> {
        StatusCode::from_repr(code as usize)
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
