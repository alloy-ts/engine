use std::cell::Cell;

/// A single-threaded, fixed-width runtime register for values that fit in 64
/// bits. `Cell` permits mutation through a shared frame/slot reference without
/// allocating or creating Rust borrow guards.
#[repr(transparent)]
pub struct ValueSlot(Cell<u64>);

impl ValueSlot {
    pub const fn new() -> Self {
        Self(Cell::new(0))
    }

    #[inline(always)]
    pub fn set_u64(&self, value: u64) {
        self.0.set(value);
    }

    #[inline(always)]
    pub fn get_u64(&self) -> u64 {
        self.0.get()
    }

    #[inline(always)]
    pub fn set_i64(&self, value: i64) {
        self.set_u64(value as u64);
    }

    #[inline(always)]
    pub fn get_i64(&self) -> i64 {
        self.get_u64() as i64
    }

    #[inline(always)]
    pub fn set_f64(&self, value: f64) {
        self.set_u64(value.to_bits());
    }

    #[inline(always)]
    pub fn get_f64(&self) -> f64 {
        f64::from_bits(self.get_u64())
    }

    #[inline(always)]
    pub fn set_i32(&self, value: i32) {
        self.set_u64(value as u32 as u64);
    }

    #[inline(always)]
    pub fn get_i32(&self) -> i32 {
        self.get_u64() as u32 as i32
    }

    #[inline(always)]
    pub fn set_bool(&self, value: bool) {
        self.set_u64(u64::from(value));
    }

    #[inline(always)]
    pub fn get_bool(&self) -> bool {
        self.get_u64() != 0
    }
}

impl Default for ValueSlot {
    fn default() -> Self {
        Self::new()
    }
}

/// Two-cell extension for inline i128/u128 values. It intentionally does not
/// pretend to be thread-safe; use an atomic or synchronized representation for
/// a runtime frame shared across worker threads.
#[repr(C)]
pub struct ExtendedValueSlot {
    low: Cell<u64>,
    high: Cell<u64>,
}

impl ExtendedValueSlot {
    pub const fn new() -> Self {
        Self {
            low: Cell::new(0),
            high: Cell::new(0),
        }
    }

    #[inline(always)]
    pub fn set_u128(&self, value: u128) {
        self.low.set(value as u64);
        self.high.set((value >> 64) as u64);
    }

    #[inline(always)]
    pub fn get_u128(&self) -> u128 {
        u128::from(self.low.get()) | (u128::from(self.high.get()) << 64)
    }

    #[inline(always)]
    pub fn set_i128(&self, value: i128) {
        self.set_u128(value as u128);
    }

    #[inline(always)]
    pub fn get_i128(&self) -> i128 {
        self.get_u128() as i128
    }
}

impl Default for ExtendedValueSlot {
    fn default() -> Self {
        Self::new()
    }
}
