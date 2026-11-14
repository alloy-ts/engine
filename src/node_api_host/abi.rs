//! C-layout declarations for the embedded Node-API bridge.
//!
//! These declarations intentionally do not export ambient `napi_*` symbols.
//! A dedicated bridge library will bind them to the safe host substrate after
//! symbol routing and library ownership are implemented.

use std::ffi::{c_char, c_void};

/// Stable Node-API version targeted by the first bridge.
pub const NAPI_VERSION: u32 = 8;

/// `napi_status` from the stable Node-API headers.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NapiStatus {
    Ok = 0,
    InvalidArg = 1,
    ObjectExpected = 2,
    StringExpected = 3,
    NameExpected = 4,
    FunctionExpected = 5,
    NumberExpected = 6,
    BooleanExpected = 7,
    ArrayExpected = 8,
    GenericFailure = 9,
    PendingException = 10,
    Cancelled = 11,
    EscapeCalledTwice = 12,
    HandleScopeMismatch = 13,
    CallbackScopeMismatch = 14,
    QueueFull = 15,
    Closing = 16,
    BigintExpected = 17,
    DateExpected = 18,
    ArraybufferExpected = 19,
    DetachableArraybufferExpected = 20,
    WouldDeadlock = 21,
    NoExternalBuffersAllowed = 22,
    CannotRunJs = 23,
}

/// `napi_valuetype` from the stable Node-API headers.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NapiValueType {
    Undefined = 0,
    Null = 1,
    Boolean = 2,
    Number = 3,
    String = 4,
    Symbol = 5,
    Object = 6,
    Function = 7,
    External = 8,
    Bigint = 9,
}

/// Opaque C types. Their storage is owned by the future bridge library.
#[repr(C)]
pub struct NapiEnv {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NapiValue {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NapiHandleScope {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NapiExtendedErrorInfo {
    pub error_message: *const c_char,
    pub engine_reserved: *mut c_void,
    pub engine_error_code: u32,
    pub error_code: NapiStatus,
}

pub type NapiGetVersion = unsafe extern "C" fn(env: *mut NapiEnv, result: *mut u32) -> NapiStatus;
pub type NapiGetLastErrorInfo = unsafe extern "C" fn(
    env: *mut NapiEnv,
    result: *mut *const NapiExtendedErrorInfo,
) -> NapiStatus;
pub type NapiGetUndefined =
    unsafe extern "C" fn(env: *mut NapiEnv, result: *mut *mut NapiValue) -> NapiStatus;
pub type NapiGetNull = NapiGetUndefined;
pub type NapiGetBoolean =
    unsafe extern "C" fn(env: *mut NapiEnv, value: bool, result: *mut *mut NapiValue) -> NapiStatus;
pub type NapiCreateDouble =
    unsafe extern "C" fn(env: *mut NapiEnv, value: f64, result: *mut *mut NapiValue) -> NapiStatus;
pub type NapiCreateStringUtf8 = unsafe extern "C" fn(
    env: *mut NapiEnv,
    value: *const c_char,
    length: usize,
    result: *mut *mut NapiValue,
) -> NapiStatus;
pub type NapiCreateStringUtf16 = unsafe extern "C" fn(
    env: *mut NapiEnv,
    value: *const u16,
    length: usize,
    result: *mut *mut NapiValue,
) -> NapiStatus;
pub type NapiTypeof = unsafe extern "C" fn(
    env: *mut NapiEnv,
    value: *mut NapiValue,
    result: *mut NapiValueType,
) -> NapiStatus;
pub type NapiOpenHandleScope =
    unsafe extern "C" fn(env: *mut NapiEnv, result: *mut *mut NapiHandleScope) -> NapiStatus;
pub type NapiCloseHandleScope =
    unsafe extern "C" fn(env: *mut NapiEnv, scope: *mut NapiHandleScope) -> NapiStatus;

impl From<super::Status> for NapiStatus {
    fn from(status: super::Status) -> Self {
        match status {
            super::Status::Ok => Self::Ok,
            super::Status::InvalidArg => Self::InvalidArg,
            super::Status::HandleScopeMismatch => Self::HandleScopeMismatch,
            super::Status::PendingException => Self::PendingException,
            super::Status::StringEncoding | super::Status::WrongThread => Self::GenericFailure,
            super::Status::Closing => Self::Closing,
        }
    }
}

impl From<super::ValueType> for NapiValueType {
    fn from(value_type: super::ValueType) -> Self {
        match value_type {
            super::ValueType::Undefined => Self::Undefined,
            super::ValueType::Null => Self::Null,
            super::ValueType::Boolean => Self::Boolean,
            super::ValueType::Number => Self::Number,
            super::ValueType::String => Self::String,
            super::ValueType::Object => Self::Object,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_discriminants_match_node_api() {
        assert_eq!(NapiStatus::Ok as i32, 0);
        assert_eq!(NapiStatus::PendingException as i32, 10);
        assert_eq!(NapiStatus::Closing as i32, 16);
        assert_eq!(NapiStatus::CannotRunJs as i32, 23);
        assert_eq!(NapiValueType::Undefined as i32, 0);
        assert_eq!(NapiValueType::Object as i32, 6);
        assert_eq!(NapiValueType::Bigint as i32, 9);
    }

    #[test]
    fn internal_statuses_have_explicit_abi_mappings() {
        assert_eq!(
            NapiStatus::from(super::super::Status::WrongThread),
            NapiStatus::GenericFailure
        );
        assert_eq!(
            NapiStatus::from(super::super::Status::HandleScopeMismatch),
            NapiStatus::HandleScopeMismatch
        );
        assert_eq!(
            NapiValueType::from(super::super::ValueType::Object),
            NapiValueType::Object
        );
    }

    #[test]
    fn declarations_are_pointer_sized_and_c_layout() {
        assert_eq!(size_of::<*mut NapiEnv>(), size_of::<usize>());
        assert_eq!(size_of::<*mut NapiValue>(), size_of::<usize>());
        assert_eq!(size_of::<NapiStatus>(), size_of::<i32>());
        assert_eq!(size_of::<NapiValueType>(), size_of::<i32>());
    }
}
