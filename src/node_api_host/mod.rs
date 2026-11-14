//! Safe substrate for Alloy's future embedded Node-API host.
//!
//! This module intentionally exports no C ABI symbols. The eventual ABI layer
//! will validate opaque pointers and delegate to these environment operations.

pub mod abi;

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, ThreadId};

use crate::runtime::{Realm, Value};

static NEXT_ENVIRONMENT_ID: AtomicU64 = AtomicU64::new(1);

/// Stable Node-API status values used by the first host slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    InvalidArg,
    HandleScopeMismatch,
    PendingException,
    StringEncoding,
    Closing,
    WrongThread,
}

/// Node-API's observable `napi_valuetype` categories supported by the MVP.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    Undefined,
    Null,
    Boolean,
    Number,
    String,
    Object,
}

/// A validated local handle token. It is not a pointer to runtime storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueHandle {
    environment: u64,
    slot: usize,
    generation: u64,
}

/// A handle-scope token owned by exactly one environment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandleScope {
    environment: u64,
    id: u64,
}

#[derive(Debug)]
struct HandleSlot {
    generation: u64,
    value: Option<Value>,
}

#[derive(Debug)]
struct ScopeFrame {
    id: u64,
    handles: Vec<usize>,
}

/// Per-module Node-API state bound to a Realm and its owner thread.
#[derive(Debug)]
pub struct Environment {
    id: u64,
    owner: ThreadId,
    realm: Realm,
    handles: Vec<HandleSlot>,
    free_handles: Vec<usize>,
    scopes: Vec<ScopeFrame>,
    next_scope_id: u64,
    pending_exception: Option<Value>,
    closing: bool,
    last_error: Status,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(Realm::new())
    }
}

impl Environment {
    pub fn new(realm: Realm) -> Self {
        Self {
            id: NEXT_ENVIRONMENT_ID.fetch_add(1, Ordering::Relaxed),
            owner: thread::current().id(),
            realm,
            handles: Vec::new(),
            free_handles: Vec::new(),
            scopes: vec![ScopeFrame {
                id: 0,
                handles: Vec::new(),
            }],
            next_scope_id: 1,
            pending_exception: None,
            closing: false,
            last_error: Status::Ok,
        }
    }

    pub fn realm(&self) -> &Realm {
        &self.realm
    }

    pub fn last_error(&self) -> Status {
        self.last_error
    }

    pub fn begin_closing(&mut self) {
        self.closing = true;
    }

    pub fn open_handle_scope(&mut self) -> Result<HandleScope, Status> {
        self.check_available(false)?;
        let scope = HandleScope {
            environment: self.id,
            id: self.next_scope_id,
        };
        self.next_scope_id += 1;
        self.scopes.push(ScopeFrame {
            id: scope.id,
            handles: Vec::new(),
        });
        self.succeed();
        Ok(scope)
    }

    pub fn close_handle_scope(&mut self, scope: HandleScope) -> Result<(), Status> {
        self.check_available(false)?;
        if scope.environment != self.id
            || self.scopes.len() == 1
            || self.scopes.last().map(|frame| frame.id) != Some(scope.id)
        {
            return self.fail(Status::HandleScopeMismatch);
        }
        let frame = self.scopes.pop().expect("root scope remains open");
        for index in frame.handles {
            let slot = &mut self.handles[index];
            slot.value = None;
            slot.generation = slot.generation.wrapping_add(1);
            self.free_handles.push(index);
        }
        self.succeed();
        Ok(())
    }

    pub fn get_undefined(&mut self) -> Result<ValueHandle, Status> {
        self.create_handle(Value::Undefined)
    }

    pub fn get_null(&mut self) -> Result<ValueHandle, Status> {
        self.create_handle(Value::Null)
    }

    pub fn get_boolean(&mut self, value: bool) -> Result<ValueHandle, Status> {
        self.create_handle(Value::Boolean(value))
    }

    pub fn create_double(&mut self, value: f64) -> Result<ValueHandle, Status> {
        self.create_handle(Value::Number(value))
    }

    pub fn create_string_utf8(&mut self, value: &str) -> Result<ValueHandle, Status> {
        self.check_available(false)?;
        let value = self.realm.alloc_string(value);
        self.allocate_handle(value)
    }

    /// Creates a string from UTF-16 when it can be represented by the current
    /// scalar-valid UTF-8 Realm heap. Lone surrogates remain an explicit MVP
    /// deviation until the runtime adopts UTF-16 or WTF-8 storage.
    pub fn create_string_utf16(&mut self, value: &[u16]) -> Result<ValueHandle, Status> {
        self.check_available(false)?;
        let string = String::from_utf16(value).map_err(|_| {
            self.last_error = Status::StringEncoding;
            Status::StringEncoding
        })?;
        let value = self.realm.alloc_string(string);
        self.allocate_handle(value)
    }

    pub fn value_type(&mut self, handle: ValueHandle) -> Result<ValueType, Status> {
        self.check_available(false)?;
        let value = self.resolve(handle)?;
        let kind = match value {
            Value::Undefined => ValueType::Undefined,
            Value::Null => ValueType::Null,
            Value::Boolean(_) => ValueType::Boolean,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Error(_) => ValueType::Object,
        };
        self.succeed();
        Ok(kind)
    }

    pub fn throw(&mut self, handle: ValueHandle) -> Result<(), Status> {
        self.check_available(false)?;
        let value = self.resolve(handle)?;
        self.pending_exception = Some(value);
        self.succeed();
        Ok(())
    }

    pub fn is_exception_pending(&self) -> bool {
        self.pending_exception.is_some()
    }

    pub fn get_and_clear_last_exception(&mut self) -> Result<Option<ValueHandle>, Status> {
        self.check_thread_and_lifecycle()?;
        let Some(value) = self.pending_exception.take() else {
            self.succeed();
            return Ok(None);
        };
        self.allocate_handle(value).map(Some)
    }

    fn create_handle(&mut self, value: Value) -> Result<ValueHandle, Status> {
        self.check_available(false)?;
        self.allocate_handle(value)
    }

    fn allocate_handle(&mut self, value: Value) -> Result<ValueHandle, Status> {
        let index = if let Some(index) = self.free_handles.pop() {
            self.handles[index].value = Some(value);
            index
        } else {
            self.handles.push(HandleSlot {
                generation: 0,
                value: Some(value),
            });
            self.handles.len() - 1
        };
        self.scopes
            .last_mut()
            .expect("environment always has a root scope")
            .handles
            .push(index);
        let handle = ValueHandle {
            environment: self.id,
            slot: index,
            generation: self.handles[index].generation,
        };
        self.succeed();
        Ok(handle)
    }

    fn resolve(&mut self, handle: ValueHandle) -> Result<Value, Status> {
        if handle.environment != self.id {
            return self.fail(Status::InvalidArg);
        }
        let Some(slot) = self.handles.get(handle.slot) else {
            return self.fail(Status::InvalidArg);
        };
        if slot.generation != handle.generation {
            return self.fail(Status::InvalidArg);
        }
        match slot.value {
            Some(value) => Ok(value),
            None => self.fail(Status::InvalidArg),
        }
    }

    fn check_available(&mut self, allow_pending: bool) -> Result<(), Status> {
        self.check_thread_and_lifecycle()?;
        if !allow_pending && self.pending_exception.is_some() {
            return self.fail(Status::PendingException);
        }
        Ok(())
    }

    fn check_thread_and_lifecycle(&mut self) -> Result<(), Status> {
        if thread::current().id() != self.owner {
            return self.fail(Status::WrongThread);
        }
        if self.closing {
            return self.fail(Status::Closing);
        }
        Ok(())
    }

    fn succeed(&mut self) {
        self.last_error = Status::Ok;
    }

    fn fail<T>(&mut self, status: Status) -> Result<T, Status> {
        self.last_error = status;
        Err(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_and_strings_are_realm_values() {
        let mut env = Environment::default();
        let values = [
            (env.get_undefined().unwrap(), ValueType::Undefined),
            (env.get_null().unwrap(), ValueType::Null),
            (env.get_boolean(true).unwrap(), ValueType::Boolean),
            (env.create_double(-0.0).unwrap(), ValueType::Number),
            (env.create_string_utf8("héllo").unwrap(), ValueType::String),
            (
                env.create_string_utf16(&[0xD83E, 0xDD8A]).unwrap(),
                ValueType::String,
            ),
        ];
        for (value, expected) in values {
            assert_eq!(env.value_type(value), Ok(expected));
        }
        assert_eq!(
            env.create_string_utf16(&[0xD800]),
            Err(Status::StringEncoding)
        );
    }

    #[test]
    fn closing_a_scope_invalidates_handles_and_reuses_with_new_generation() {
        let mut env = Environment::default();
        let scope = env.open_handle_scope().unwrap();
        let stale = env.create_double(1.0).unwrap();
        env.close_handle_scope(scope).unwrap();
        assert_eq!(env.value_type(stale), Err(Status::InvalidArg));
        let replacement = env.create_double(2.0).unwrap();
        assert_eq!(stale.slot, replacement.slot);
        assert_ne!(stale.generation, replacement.generation);
    }

    #[test]
    fn scopes_close_in_stack_order_and_reject_foreign_tokens() {
        let mut env = Environment::default();
        let outer = env.open_handle_scope().unwrap();
        let inner = env.open_handle_scope().unwrap();
        assert_eq!(
            env.close_handle_scope(outer),
            Err(Status::HandleScopeMismatch)
        );
        env.close_handle_scope(inner).unwrap();
        env.close_handle_scope(outer).unwrap();

        let mut other = Environment::default();
        let foreign = other.open_handle_scope().unwrap();
        assert_eq!(
            env.close_handle_scope(foreign),
            Err(Status::HandleScopeMismatch)
        );
    }

    #[test]
    fn handles_cannot_cross_environments() {
        let mut first = Environment::default();
        let value = first.get_null().unwrap();
        let mut second = Environment::default();
        assert_eq!(second.value_type(value), Err(Status::InvalidArg));
    }

    #[test]
    fn pending_exception_blocks_ordinary_calls_and_can_be_cleared() {
        let mut env = Environment::default();
        let thrown = env.create_string_utf8("boom").unwrap();
        env.throw(thrown).unwrap();
        assert!(env.is_exception_pending());
        assert_eq!(env.get_null(), Err(Status::PendingException));
        assert_eq!(env.last_error(), Status::PendingException);

        let recovered = env.get_and_clear_last_exception().unwrap().unwrap();
        assert!(!env.is_exception_pending());
        assert_eq!(env.value_type(recovered), Ok(ValueType::String));
    }

    #[test]
    fn closing_environment_rejects_new_work() {
        let mut env = Environment::default();
        env.begin_closing();
        assert_eq!(env.get_undefined(), Err(Status::Closing));
    }

    #[test]
    fn operations_reject_a_non_owner_thread() {
        let env = Environment::default();
        let (status, last_error) = std::thread::spawn(move || {
            let mut env = env;
            (env.get_undefined(), env.last_error())
        })
        .join()
        .unwrap();
        assert_eq!(status, Err(Status::WrongThread));
        assert_eq!(last_error, Status::WrongThread);
    }
}
