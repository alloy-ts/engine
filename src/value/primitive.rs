use std::cell::Cell;

/// Runtime primitive categories used by the in-memory Value representation.
///
/// `Integer` is an addon-level category; JavaScript's native `typeof` still
/// reports Integer wrapper objects as `"object"` unless source code is passed
/// through an explicit type inspection helper or transform.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveKind {
    Null,
    Undefined,
    Boolean,
    Number,
    Integer,
    String,
    Symbol,
}

impl PrimitiveKind {
    #[inline]
    pub const fn typeof_name(self) -> &'static str {
        match self {
            Self::Null => "object",
            Self::Undefined => "undefined",
            Self::Boolean => "boolean",
            Self::Number => "number",
            Self::Integer => "integer",
            Self::String => "string",
            Self::Symbol => "symbol",
        }
    }

    #[inline]
    pub const fn wrapper_name(self) -> Option<&'static str> {
        match self {
            Self::Boolean => Some("Boolean"),
            Self::Number => Some("Number"),
            Self::Integer => Some("Integer"),
            Self::String => Some("String"),
            Self::Symbol => Some("Symbol"),
            Self::Null | Self::Undefined => None,
        }
    }
}

/// A 16-byte tagged runtime value for primitives whose payload fits in 64
/// bits. The payload is copied in and out through `Cell`; it never exposes an
/// interior reference and is not thread-safe by design.
#[repr(C)]
pub struct PrimitiveCell {
    kind: Cell<PrimitiveKind>,
    payload: Cell<u64>,
}

impl PrimitiveCell {
    pub const fn new(kind: PrimitiveKind) -> Self {
        Self {
            kind: Cell::new(kind),
            payload: Cell::new(0),
        }
    }

    #[inline(always)]
    pub fn kind(&self) -> PrimitiveKind {
        self.kind.get()
    }

    #[inline(always)]
    pub fn set_kind(&self, kind: PrimitiveKind) {
        self.kind.set(kind);
    }

    #[inline(always)]
    pub fn set_bool(&self, value: bool) {
        self.kind.set(PrimitiveKind::Boolean);
        self.payload.set(u64::from(value));
    }

    #[inline(always)]
    pub fn get_bool(&self) -> bool {
        debug_assert_eq!(self.kind(), PrimitiveKind::Boolean);
        self.payload.get() != 0
    }

    #[inline(always)]
    pub fn set_number(&self, value: f64) {
        self.kind.set(PrimitiveKind::Number);
        self.payload.set(value.to_bits());
    }

    #[inline(always)]
    pub fn get_number(&self) -> f64 {
        debug_assert_eq!(self.kind(), PrimitiveKind::Number);
        f64::from_bits(self.payload.get())
    }

    #[inline(always)]
    pub fn set_integer(&self, value: i64) {
        self.kind.set(PrimitiveKind::Integer);
        self.payload.set(value as u64);
    }

    #[inline(always)]
    pub fn get_integer(&self) -> i64 {
        debug_assert_eq!(self.kind(), PrimitiveKind::Integer);
        self.payload.get() as i64
    }

    /// Stores an opaque runtime handle for String or Symbol data. The handle is
    /// owned by the surrounding runtime/arena, not by this Cell.
    #[inline(always)]
    pub fn set_handle(&self, kind: PrimitiveKind, handle: u64) {
        debug_assert!(matches!(
            kind,
            PrimitiveKind::String | PrimitiveKind::Symbol
        ));
        self.kind.set(kind);
        self.payload.set(handle);
    }

    #[inline(always)]
    pub fn handle(&self) -> u64 {
        debug_assert!(matches!(
            self.kind(),
            PrimitiveKind::String | PrimitiveKind::Symbol
        ));
        self.payload.get()
    }
}

impl Default for PrimitiveCell {
    fn default() -> Self {
        Self::new(PrimitiveKind::Undefined)
    }
}

/// A tagged 128-bit Integer value. The tag is separate from the two-cell
/// payload so all i128/u128 bits remain inline and no heap indirection is
/// required.
#[repr(C)]
pub struct WidePrimitiveCell {
    kind: Cell<PrimitiveKind>,
    low: Cell<u64>,
    high: Cell<u64>,
}

impl WidePrimitiveCell {
    pub const fn new(kind: PrimitiveKind) -> Self {
        Self {
            kind: Cell::new(kind),
            low: Cell::new(0),
            high: Cell::new(0),
        }
    }

    #[inline(always)]
    pub fn kind(&self) -> PrimitiveKind {
        self.kind.get()
    }

    #[inline(always)]
    pub fn set_integer(&self, value: i128) {
        self.kind.set(PrimitiveKind::Integer);
        let bits = value as u128;
        self.low.set(bits as u64);
        self.high.set((bits >> 64) as u64);
    }

    #[inline(always)]
    pub fn get_integer(&self) -> i128 {
        debug_assert_eq!(self.kind(), PrimitiveKind::Integer);
        (u128::from(self.low.get()) | (u128::from(self.high.get()) << 64)) as i128
    }
}

#[cfg(test)]
mod tests {
    use super::{PrimitiveCell, PrimitiveKind, WidePrimitiveCell};

    #[test]
    fn maps_primitive_types_and_wrappers() {
        assert_eq!(PrimitiveKind::Null.typeof_name(), "object");
        assert_eq!(PrimitiveKind::Undefined.typeof_name(), "undefined");
        assert_eq!(PrimitiveKind::Integer.typeof_name(), "integer");
        assert_eq!(PrimitiveKind::Integer.wrapper_name(), Some("Integer"));
        assert_eq!(PrimitiveKind::Null.wrapper_name(), None);
    }

    #[test]
    fn stores_fixed_width_values_without_heap_allocation() {
        let value = PrimitiveCell::default();
        value.set_number(2.5);
        assert_eq!(value.kind(), PrimitiveKind::Number);
        assert_eq!(value.get_number(), 2.5);

        value.set_bool(true);
        assert!(value.get_bool());

        let wide = WidePrimitiveCell::new(PrimitiveKind::Undefined);
        let source = i128::MIN + 17;
        wide.set_integer(source);
        assert_eq!(wide.get_integer(), source);

        assert_eq!(std::mem::size_of::<PrimitiveCell>(), 16);
        assert_eq!(std::mem::size_of::<WidePrimitiveCell>(), 24);
    }
}
