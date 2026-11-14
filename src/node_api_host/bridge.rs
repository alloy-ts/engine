//! Non-exported owner for the raw Node-API bridge boundary.
//!
//! The owner keeps every opaque token allocation alive until it is dropped, so
//! pointer identity can be checked without dereferencing untrusted pointers.
//! Standard `napi_*` symbols are deliberately not exported from this module.

use std::ffi::c_char;
use std::ptr;

use super::abi::{
    NAPI_VERSION, NapiEnv, NapiHandleScope, NapiStatus, NapiValue, NapiValueType, napi_env,
    napi_handle_scope, napi_value,
};
use super::{Environment, HandleScope, Status, ValueHandle};

/// Node-API's sentinel for a NUL-terminated input string.
pub const NAPI_AUTO_LENGTH: usize = usize::MAX;

#[repr(C)]
#[derive(Debug)]
struct EnvironmentToken {
    marker: u8,
}

#[repr(C)]
#[derive(Debug)]
struct ValueToken {
    handle: ValueHandle,
}

#[repr(C)]
#[derive(Debug)]
struct ScopeToken {
    scope: HandleScope,
    open: bool,
}

/// Owns one environment and all opaque allocations handed to native code.
#[derive(Debug)]
pub struct BridgeOwner {
    environment: Environment,
    environment_token: Box<EnvironmentToken>,
    // Each allocation must remain at a stable address while the Vec grows.
    #[allow(clippy::vec_box)]
    values: Vec<Box<ValueToken>>,
    #[allow(clippy::vec_box)]
    scopes: Vec<Box<ScopeToken>>,
}

impl Default for BridgeOwner {
    fn default() -> Self {
        Self::new(Environment::default())
    }
}

impl BridgeOwner {
    pub fn new(environment: Environment) -> Self {
        Self {
            environment,
            environment_token: Box::new(EnvironmentToken { marker: 0 }),
            values: Vec::new(),
            scopes: Vec::new(),
        }
    }

    pub fn env(&self) -> napi_env {
        ptr::from_ref(self.environment_token.as_ref())
            .cast_mut()
            .cast::<NapiEnv>()
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    /// Implements the bridge behavior for `napi_get_version` without exporting
    /// that ambient symbol from the outer addon.
    ///
    /// # Safety
    ///
    /// A non-null `result` must point to writable `u32` storage.
    pub unsafe fn get_version(&mut self, env: napi_env, result: *mut u32) -> NapiStatus {
        if let Err(status) = self.validate_env(env) {
            return status;
        }
        if result.is_null() {
            return self.fail(Status::InvalidArg);
        }
        if let Err(status) = self.environment.check_basic_operation() {
            return status.into();
        }
        // SAFETY: The caller guarantees writable storage and null was checked.
        unsafe { result.write(NAPI_VERSION) };
        NapiStatus::Ok
    }

    /// # Safety
    ///
    /// A non-null `result` must point to writable `napi_value` storage.
    pub unsafe fn get_undefined(&mut self, env: napi_env, result: *mut napi_value) -> NapiStatus {
        self.create_value(env, result, Environment::get_undefined)
    }

    /// # Safety
    ///
    /// A non-null `result` must point to writable `napi_value` storage.
    pub unsafe fn get_null(&mut self, env: napi_env, result: *mut napi_value) -> NapiStatus {
        self.create_value(env, result, Environment::get_null)
    }

    /// # Safety
    ///
    /// A non-null `result` must point to writable `napi_value` storage.
    pub unsafe fn get_boolean(
        &mut self,
        env: napi_env,
        value: bool,
        result: *mut napi_value,
    ) -> NapiStatus {
        self.create_value(env, result, |environment| environment.get_boolean(value))
    }

    /// # Safety
    ///
    /// A non-null `result` must point to writable `napi_value` storage.
    pub unsafe fn create_double(
        &mut self,
        env: napi_env,
        value: f64,
        result: *mut napi_value,
    ) -> NapiStatus {
        self.create_value(env, result, |environment| environment.create_double(value))
    }

    /// # Safety
    ///
    /// `value` must reference `length` readable bytes, or a readable
    /// NUL-terminated sequence when `length == NAPI_AUTO_LENGTH`. A non-null
    /// `result` must point to writable `napi_value` storage.
    pub unsafe fn create_string_utf8(
        &mut self,
        env: napi_env,
        value: *const c_char,
        length: usize,
        result: *mut napi_value,
    ) -> NapiStatus {
        if let Err(status) = self.validate_value_output(env, result) {
            return status;
        }
        if value.is_null() {
            return self.fail(Status::InvalidArg);
        }
        let bytes = if length == NAPI_AUTO_LENGTH {
            // SAFETY: The caller promises a readable NUL-terminated sequence.
            unsafe { std::ffi::CStr::from_ptr(value) }.to_bytes()
        } else {
            // SAFETY: The caller promises `length` readable bytes.
            unsafe { std::slice::from_raw_parts(value.cast::<u8>(), length) }
        };
        let Ok(string) = std::str::from_utf8(bytes) else {
            return self.fail(Status::StringEncoding);
        };
        match self.environment.create_string_utf8(string) {
            Ok(handle) => self.write_value(handle, result),
            Err(status) => status.into(),
        }
    }

    /// # Safety
    ///
    /// `value` must reference `length` readable UTF-16 code units, or a
    /// readable NUL-terminated sequence when `length == NAPI_AUTO_LENGTH`.
    /// A non-null `result` must point to writable `napi_value` storage.
    pub unsafe fn create_string_utf16(
        &mut self,
        env: napi_env,
        value: *const u16,
        length: usize,
        result: *mut napi_value,
    ) -> NapiStatus {
        if let Err(status) = self.validate_value_output(env, result) {
            return status;
        }
        if value.is_null() {
            return self.fail(Status::InvalidArg);
        }
        let length = if length == NAPI_AUTO_LENGTH {
            let mut length = 0;
            // SAFETY: The caller promises a readable NUL-terminated sequence.
            while unsafe { value.add(length).read() } != 0 {
                length += 1;
            }
            length
        } else {
            length
        };
        // SAFETY: The caller promises `length` readable code units.
        let units = unsafe { std::slice::from_raw_parts(value, length) };
        match self.environment.create_string_utf16(units) {
            Ok(handle) => self.write_value(handle, result),
            Err(status) => status.into(),
        }
    }

    /// # Safety
    ///
    /// A non-null `result` must point to writable `NapiValueType` storage.
    pub unsafe fn type_of(
        &mut self,
        env: napi_env,
        value: napi_value,
        result: *mut NapiValueType,
    ) -> NapiStatus {
        if let Err(status) = self.validate_env(env) {
            return status;
        }
        if result.is_null() {
            return self.fail(Status::InvalidArg);
        }
        let Some(handle) = self.value_handle(value) else {
            return self.fail(Status::InvalidArg);
        };
        match self.environment.value_type(handle) {
            Ok(value_type) => {
                // SAFETY: The caller guarantees writable storage.
                unsafe { result.write(value_type.into()) };
                NapiStatus::Ok
            }
            Err(status) => status.into(),
        }
    }

    /// # Safety
    ///
    /// A non-null `result` must point to writable `napi_handle_scope` storage.
    pub unsafe fn open_handle_scope(
        &mut self,
        env: napi_env,
        result: *mut napi_handle_scope,
    ) -> NapiStatus {
        if let Err(status) = self.validate_env(env) {
            return status;
        }
        if result.is_null() {
            return self.fail(Status::InvalidArg);
        }
        match self.environment.open_handle_scope() {
            Ok(scope) => {
                let token = Box::new(ScopeToken { scope, open: true });
                let pointer = ptr::from_ref(token.as_ref())
                    .cast_mut()
                    .cast::<NapiHandleScope>();
                self.scopes.push(token);
                // SAFETY: The caller guarantees writable storage.
                unsafe { result.write(pointer) };
                NapiStatus::Ok
            }
            Err(status) => status.into(),
        }
    }

    pub fn close_handle_scope(&mut self, env: napi_env, scope: napi_handle_scope) -> NapiStatus {
        if let Err(status) = self.validate_env(env) {
            return status;
        }
        let Some(index) = self.scope_index(scope) else {
            return self.fail(Status::HandleScopeMismatch);
        };
        if !self.scopes[index].open {
            return self.fail(Status::HandleScopeMismatch);
        }
        match self
            .environment
            .close_handle_scope(self.scopes[index].scope)
        {
            Ok(()) => {
                self.scopes[index].open = false;
                NapiStatus::Ok
            }
            Err(status) => status.into(),
        }
    }

    fn create_value(
        &mut self,
        env: napi_env,
        result: *mut napi_value,
        create: impl FnOnce(&mut Environment) -> Result<ValueHandle, Status>,
    ) -> NapiStatus {
        if let Err(status) = self.validate_value_output(env, result) {
            return status;
        }
        match create(&mut self.environment) {
            Ok(handle) => self.write_value(handle, result),
            Err(status) => status.into(),
        }
    }

    fn write_value(&mut self, handle: ValueHandle, result: *mut napi_value) -> NapiStatus {
        let token = Box::new(ValueToken { handle });
        let pointer = ptr::from_ref(token.as_ref()).cast_mut().cast::<NapiValue>();
        self.values.push(token);
        // SAFETY: All callers validated the output pointer before reaching here.
        unsafe { result.write(pointer) };
        NapiStatus::Ok
    }

    fn validate_value_output(
        &mut self,
        env: napi_env,
        result: *mut napi_value,
    ) -> Result<(), NapiStatus> {
        self.validate_env(env)?;
        if result.is_null() {
            return Err(self.fail(Status::InvalidArg));
        }
        Ok(())
    }

    fn validate_env(&mut self, env: napi_env) -> Result<(), NapiStatus> {
        if env.is_null() || !ptr::eq(env, self.env()) {
            return Err(NapiStatus::InvalidArg);
        }
        Ok(())
    }

    fn value_handle(&self, value: napi_value) -> Option<ValueHandle> {
        if value.is_null() {
            return None;
        }
        self.values
            .iter()
            .find(|token| ptr::eq(value.cast::<ValueToken>(), ptr::from_ref(token.as_ref())))
            .map(|token| token.handle)
    }

    fn scope_index(&self, scope: napi_handle_scope) -> Option<usize> {
        if scope.is_null() {
            return None;
        }
        self.scopes.iter().position(|token| {
            ptr::eq(
                scope.cast::<ScopeToken>(),
                ptr::from_ref::<ScopeToken>(token.as_ref()),
            )
        })
    }

    fn fail(&mut self, status: Status) -> NapiStatus {
        self.environment.record_failure(status).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_and_foreign_environments_are_rejected_without_writing_outputs() {
        let mut owner = BridgeOwner::default();
        let mut other = BridgeOwner::default();
        let sentinel = ptr::dangling_mut::<NapiValue>();
        let mut output = sentinel;
        // SAFETY: `output` is writable; invalid environments are intentional.
        assert_eq!(
            unsafe { owner.get_undefined(ptr::null_mut(), &mut output) },
            NapiStatus::InvalidArg
        );
        assert_eq!(output, sentinel);
        // SAFETY: `output` is writable; the foreign environment is live.
        assert_eq!(
            unsafe { owner.get_undefined(other.env(), &mut output) },
            NapiStatus::InvalidArg
        );
        assert_eq!(output, sentinel);
        // Keep `other` mutably used so this test also covers distinct owners.
        assert_eq!(
            unsafe { other.get_version(other.env(), ptr::null_mut()) },
            NapiStatus::InvalidArg
        );
    }

    #[test]
    fn primitive_outputs_and_types_delegate_to_environment() {
        let mut owner = BridgeOwner::default();
        let env = owner.env();
        let mut value = ptr::null_mut();
        // SAFETY: All output pointers reference writable local storage.
        assert_eq!(
            unsafe { owner.create_double(env, -0.0, &mut value) },
            NapiStatus::Ok
        );
        let mut value_type = NapiValueType::Undefined;
        // SAFETY: `value` belongs to this owner and the output is writable.
        assert_eq!(
            unsafe { owner.type_of(env, value, &mut value_type) },
            NapiStatus::Ok
        );
        assert_eq!(value_type, NapiValueType::Number);
        let mut version = 0;
        // SAFETY: The output is writable.
        assert_eq!(
            unsafe { owner.get_version(env, &mut version) },
            NapiStatus::Ok
        );
        assert_eq!(version, NAPI_VERSION);
    }

    #[test]
    fn closed_scope_values_are_stale_and_outputs_remain_untouched() {
        let mut owner = BridgeOwner::default();
        let env = owner.env();
        let mut scope = ptr::null_mut();
        // SAFETY: The output is writable.
        assert_eq!(
            unsafe { owner.open_handle_scope(env, &mut scope) },
            NapiStatus::Ok
        );
        let mut value = ptr::null_mut();
        // SAFETY: The output is writable.
        assert_eq!(unsafe { owner.get_null(env, &mut value) }, NapiStatus::Ok);
        assert_eq!(owner.close_handle_scope(env, scope), NapiStatus::Ok);
        let mut output = NapiValueType::Function;
        // SAFETY: The stale pointer remains owned by the bridge and output is writable.
        assert_eq!(
            unsafe { owner.type_of(env, value, &mut output) },
            NapiStatus::InvalidArg
        );
        assert_eq!(output, NapiValueType::Function);
        assert_eq!(
            owner.close_handle_scope(env, scope),
            NapiStatus::HandleScopeMismatch
        );
    }

    #[test]
    fn pending_exception_blocks_creation_without_touching_output() {
        let mut owner = BridgeOwner::default();
        let env = owner.env();
        let mut thrown = ptr::null_mut();
        // SAFETY: The output is writable.
        assert_eq!(
            unsafe { owner.get_boolean(env, true, &mut thrown) },
            NapiStatus::Ok
        );
        let handle = owner.value_handle(thrown).unwrap();
        owner.environment.throw(handle).unwrap();
        let sentinel = ptr::dangling_mut::<NapiValue>();
        let mut output = sentinel;
        // SAFETY: The output is writable.
        assert_eq!(
            unsafe { owner.get_null(env, &mut output) },
            NapiStatus::PendingException
        );
        assert_eq!(output, sentinel);
    }

    #[test]
    fn strings_validate_null_and_encoding() {
        let mut owner = BridgeOwner::default();
        let env = owner.env();
        let mut output = ptr::null_mut();
        // SAFETY: Null input is intentional and output is writable.
        assert_eq!(
            unsafe { owner.create_string_utf8(env, ptr::null(), 0, &mut output) },
            NapiStatus::InvalidArg
        );
        let invalid = [0xFF_u8];
        // SAFETY: `invalid` contains one readable byte and output is writable.
        assert_eq!(
            unsafe {
                owner.create_string_utf8(
                    env,
                    invalid.as_ptr().cast::<c_char>(),
                    invalid.len(),
                    &mut output,
                )
            },
            NapiStatus::GenericFailure
        );
        let surrogate = [0xD800_u16];
        // SAFETY: `surrogate` contains one readable code unit and output is writable.
        assert_eq!(
            unsafe { owner.create_string_utf16(env, surrogate.as_ptr(), 1, &mut output) },
            NapiStatus::GenericFailure
        );
    }

    #[test]
    fn foreign_scopes_are_rejected() {
        let mut owner = BridgeOwner::default();
        let mut other = BridgeOwner::default();
        let mut scope = ptr::null_mut();
        // SAFETY: The output is writable.
        assert_eq!(
            unsafe { other.open_handle_scope(other.env(), &mut scope) },
            NapiStatus::Ok
        );
        assert_eq!(
            owner.close_handle_scope(owner.env(), scope),
            NapiStatus::HandleScopeMismatch
        );
    }

    #[test]
    fn bridge_delegates_owner_thread_enforcement_without_writing_output() {
        let owner = BridgeOwner::default();
        let status = std::thread::spawn(move || {
            let mut owner = owner;
            let mut version = 123;
            // SAFETY: The output is writable and the environment token remains live.
            let status = unsafe { owner.get_version(owner.env(), &mut version) };
            (status, version)
        })
        .join()
        .unwrap();
        assert_eq!(status, (NapiStatus::GenericFailure, 123));
    }
}
