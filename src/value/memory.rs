use std::cell::Cell;

use num_bigint::{BigInt, Sign};
use num_traits::ToPrimitive;

const NAN_BASE: u64 = 0x7ff8_0000_0000_0000;
const TAG_BITS: u64 = 0x0007_0000_0000_0000;
const PAYLOAD: u64 = 0x0000_ffff_ffff_ffff;
const STRING_LITERAL_FLAG: u64 = 1u64 << 47;
const MIN_INLINE: i64 = -(1i64 << 47);
const MAX_INLINE: i64 = (1i64 << 47) - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum HeapKind {
    BigInt = 1,
    BigNum = 2,
    String = 3,
    Object = 4,
    Number64 = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum BigNumFormat {
    Float80 = 1,
    Float128 = 2,
    Decimal128 = 3,
    DoubleDouble = 4,
    Custom = 5,
}

/// Eight bytes at the beginning of every chunk. Payload bytes follow at offset 8.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct HeapHeader {
    pub kind: HeapKind,
    pub flags: u8,
    pub sub_kind: u16,
    pub byte_len: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MemoryError {
    InvalidHandle,
    WrongKind,
    InvalidPayload,
    Full,
    LengthOverflow,
}

/// One allocation contains the header and the payload. Integer limbs use
/// little-endian u64 words and objects contain u64 cells. This generic chunk
/// can also hold raw UTF-8 bytes; language-level strings with encoding and
/// UTF-16 code-unit metadata use `value::string::StringBlock`.
/// The header is encoded into the first eight bytes; no Vec inside the chunk.
pub struct HeapChunk {
    bytes: Box<[u8]>,
}

impl HeapChunk {
    pub fn new(
        kind: HeapKind,
        flags: u8,
        sub_kind: u16,
        payload: &[u8],
    ) -> Result<Self, MemoryError> {
        let len = u32::try_from(payload.len()).map_err(|_| MemoryError::LengthOverflow)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(
                payload
                    .len()
                    .checked_add(8)
                    .ok_or(MemoryError::LengthOverflow)?,
            )
            .map_err(|_| MemoryError::Full)?;
        bytes.extend_from_slice(&[kind as u8, flags]);
        bytes.extend_from_slice(&sub_kind.to_le_bytes());
        bytes.extend_from_slice(&len.to_le_bytes());
        bytes.extend_from_slice(payload);
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
        })
    }

    pub fn kind(&self) -> HeapKind {
        match self.bytes[0] {
            1 => HeapKind::BigInt,
            2 => HeapKind::BigNum,
            3 => HeapKind::String,
            4 => HeapKind::Object,
            5 => HeapKind::Number64,
            _ => unreachable!(),
        }
    }
    pub fn flags(&self) -> u8 {
        self.bytes[1]
    }
    pub fn sub_kind(&self) -> u16 {
        u16::from_le_bytes([self.bytes[2], self.bytes[3]])
    }
    pub fn payload(&self) -> &[u8] {
        &self.bytes[8..]
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum Tag {
    Null = 1,
    Bool = 2,
    Int = 3,
    BigInt = 4,
    BigNum = 5,
    String = 6,
    Object = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellKind {
    Number,
    Null,
    Undefined,
    Boolean,
    Integer,
    BigInt,
    BigNum,
    String,
    StringLiteral,
    Object,
}

/// Values are self-describing in exactly eight bytes. Tags use bits 50..48
/// of a canonical quiet NaN. All NaN numbers canonicalize at ingress.
/// Heap payloads are handles, not raw pointers.
#[repr(transparent)]
pub struct ScalarCell(Cell<u64>);

/// Optional 32-bit register representation. The high nibble is a tag and the
/// low 28 bits hold an immediate integer or a one-based arena handle. Exact
/// f64 values use an eight-byte Number64 chunk; no precision is discarded.
#[repr(transparent)]
pub struct CompactCell(Cell<u32>);

const COMPACT_PAYLOAD: u32 = 0x0fff_ffff;
const COMPACT_MIN: i32 = -(1 << 27);
const COMPACT_MAX: i32 = (1 << 27) - 1;
#[cfg(target_pointer_width = "32")]
pub const SML_INT_BITS: u32 = 16;
#[cfg(not(target_pointer_width = "32"))]
pub const SML_INT_BITS: u32 = 32;
#[cfg(target_pointer_width = "32")]
pub const SML_NUM_BITS: u32 = 16;
#[cfg(not(target_pointer_width = "32"))]
pub const SML_NUM_BITS: u32 = 32;
#[cfg(target_pointer_width = "32")]
pub const NORMAL_CELL_BITS: u32 = 32;
#[cfg(not(target_pointer_width = "32"))]
pub const NORMAL_CELL_BITS: u32 = 64;
pub const SML_INT_MIN: i32 = (-(1i64 << (SML_INT_BITS - 1))) as i32;
pub const SML_INT_MAX: i32 = ((1i64 << (SML_INT_BITS - 1)) - 1) as i32;

impl CompactCell {
    pub const fn null() -> Self {
        Self(Cell::new(1u32 << 28))
    }
    pub const fn undefined() -> Self {
        Self(Cell::new(2u32 << 28))
    }
    pub const fn boolean(value: bool) -> Self {
        Self(Cell::new((3u32 << 28) | value as u32))
    }
    pub fn integer(value: i32) -> Option<Self> {
        (COMPACT_MIN..=COMPACT_MAX)
            .contains(&value)
            .then(|| Self(Cell::new((4u32 << 28) | ((value as u32) & COMPACT_PAYLOAD))))
    }
    pub fn unsigned_integer(value: u16) -> Self {
        Self(Cell::new((11u32 << 28) | value as u32))
    }
    fn heap(tag: u32, index: usize) -> Result<Self, MemoryError> {
        let handle = u32::try_from(index).map_err(|_| MemoryError::Full)?;
        if handle >= COMPACT_PAYLOAD {
            return Err(MemoryError::Full);
        }
        Ok(Self(Cell::new((tag << 28) | (handle + 1))))
    }
    pub fn bits(&self) -> u32 {
        self.0.get()
    }
    pub fn schema_type(&self) -> &'static str {
        match self.bits() >> 28 {
            1 => "null",
            2 => "undefined",
            3 => "boolean",
            4 | 5 | 11 => "integer",
            6 | 9 | 10 => "number",
            7 => "string",
            8 => "object",
            _ => "undefined",
        }
    }
    pub fn inline_integer(&self) -> Option<i32> {
        (self.bits() >> 28 == 4).then(|| (((self.bits() & COMPACT_PAYLOAD) << 4) as i32) >> 4)
    }
    pub fn inline_unsigned_integer(&self) -> Option<u16> {
        (self.bits() >> 28 == 11).then(|| (self.bits() & 0xffff) as u16)
    }
    pub fn inline_float16(&self) -> Option<u16> {
        (self.bits() >> 28 == 10).then(|| (self.bits() & 0xffff) as u16)
    }
    pub fn float16(value: u16) -> Self {
        Self(Cell::new((10u32 << 28) | value as u32))
    }
    pub fn string_literal(id: u16) -> Self {
        Self(Cell::new((7u32 << 28) | (1u32 << 27) | id as u32))
    }
    pub fn string_block(handle: u32) -> Option<Self> {
        (handle < (1u32 << 27) - 1).then(|| Self(Cell::new((7u32 << 28) | (handle + 1))))
    }
    pub fn string_literal_id(&self) -> Option<u16> {
        ((self.bits() >> 28 == 7) && (self.bits() & (1u32 << 27) != 0))
            .then(|| (self.bits() & 0xffff) as u16)
    }
    pub fn string_block_handle(&self) -> Option<u32> {
        ((self.bits() >> 28 == 7) && (self.bits() & (1u32 << 27) == 0))
            .then(|| self.bits() & COMPACT_PAYLOAD)
            .filter(|encoded| *encoded != 0)
            .map(|encoded| encoded - 1)
    }
    fn handle(&self) -> Option<usize> {
        ((5..=6).contains(&(self.bits() >> 28))
            || (8..=9).contains(&(self.bits() >> 28))
            || self.string_block_handle().is_some())
        .then(|| (self.bits() & COMPACT_PAYLOAD) as usize)
        .filter(|index| *index != 0)
        .map(|index| index - 1)
    }
}

/// A compact integer payload. Its width follows the runtime policy: i32 on
/// 64-bit targets and i16 on 32-bit targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct SmlInt {
    raw: u32,
}

impl SmlInt {
    pub fn new(value: i32) -> Result<Self, MemoryError> {
        if !(SML_INT_MIN..=SML_INT_MAX).contains(&value) {
            return Err(MemoryError::InvalidPayload);
        }
        #[cfg(target_pointer_width = "32")]
        let raw = CompactCell::integer(value)
            .ok_or(MemoryError::InvalidPayload)?
            .bits();
        #[cfg(not(target_pointer_width = "32"))]
        let raw = value as u32;
        Ok(Self { raw })
    }
    pub fn new_unsigned(value: u64) -> Result<Self, MemoryError> {
        let max = (1u64 << SML_INT_BITS) - 1;
        if value > max {
            return Err(MemoryError::InvalidPayload);
        }
        #[cfg(target_pointer_width = "32")]
        let raw = CompactCell::unsigned_integer(value as u16).bits();
        #[cfg(not(target_pointer_width = "32"))]
        let raw = value as u32;
        Ok(Self { raw })
    }
    pub fn bits(self) -> u32 {
        self.raw
    }
    pub fn signed_value(self) -> i64 {
        #[cfg(target_pointer_width = "32")]
        {
            (((self.raw & 0xffff) << 16) as i32 >> 16) as i64
        }
        #[cfg(not(target_pointer_width = "32"))]
        {
            (self.raw as i32) as i64
        }
    }
    pub fn unsigned_value(self) -> u64 {
        #[cfg(target_pointer_width = "32")]
        {
            (self.raw & 0xffff) as u64
        }
        #[cfg(not(target_pointer_width = "32"))]
        {
            self.raw as u64
        }
    }
    pub fn value(self) -> i32 {
        self.signed_value() as i32
    }
    #[cfg(target_pointer_width = "32")]
    pub fn runtime_cell(self, unsigned: bool) -> CompactCell {
        if unsigned {
            CompactCell::unsigned_integer(self.unsigned_value() as u16)
        } else {
            CompactCell::integer(self.signed_value() as i32).expect("SmlInt range fits CompactCell")
        }
    }
    #[cfg(not(target_pointer_width = "32"))]
    pub fn runtime_cell(self, unsigned: bool) -> ScalarCell {
        let value = if unsigned {
            self.unsigned_value() as i64
        } else {
            self.signed_value()
        };
        ScalarCell::inline_integer(value).expect("SmlInt range fits ScalarCell")
    }
    pub fn as_number(self, unsigned: bool) -> SmlNum {
        let value = if unsigned {
            self.unsigned_value() as f64
        } else {
            self.signed_value() as f64
        };
        SmlNum::from_f64(value).expect("SmlInt always converts to SmlNum")
    }
}

/// Reduced-precision Number payload: binary32 on 64-bit runtimes and binary16
/// on 32-bit runtimes. The exposed JavaScript primitive remains a Number.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SmlNum {
    bits: u32,
}

impl SmlNum {
    pub fn from_f64(value: f64) -> Result<Self, MemoryError> {
        #[cfg(target_pointer_width = "32")]
        let bits = CompactCell::float16(f32_to_f16(value as f32)).bits();
        #[cfg(not(target_pointer_width = "32"))]
        let bits = if value.is_nan() {
            f32::NAN.to_bits()
        } else {
            (value as f32).to_bits()
        };
        Ok(Self { bits })
    }
    pub fn from_integer(value: SmlInt) -> Self {
        value.as_number(false)
    }
    pub fn bits(self) -> u32 {
        self.bits
    }
    pub fn to_f64(self) -> f64 {
        #[cfg(target_pointer_width = "32")]
        {
            f16_to_f32((self.bits & 0xffff) as u16) as f64
        }
        #[cfg(not(target_pointer_width = "32"))]
        {
            f32::from_bits(self.bits) as f64
        }
    }
    #[cfg(target_pointer_width = "32")]
    pub fn runtime_cell(self) -> CompactCell {
        CompactCell(Cell::new(self.bits))
    }
    #[cfg(not(target_pointer_width = "32"))]
    pub fn runtime_cell(self) -> ScalarCell {
        ScalarCell::number(self.to_f64())
    }
}

#[cfg(target_pointer_width = "32")]
fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x7f_ffff;
    if exponent == 0xff {
        return sign | if mantissa == 0 { 0x7c00 } else { 0x7e00 };
    }
    let half_exponent = exponent - 127 + 15;
    if half_exponent >= 31 {
        return sign | 0x7c00;
    }
    if half_exponent <= 0 {
        if half_exponent < -10 {
            return sign;
        }
        let significand = mantissa | 0x80_0000;
        let shift = (14 - half_exponent) as u32;
        let rounding = (1u32 << (shift - 1)) - 1 + ((significand >> shift) & 1);
        return sign | ((significand + rounding) >> shift) as u16;
    }
    let rounding = 0x0fff + ((mantissa >> 13) & 1);
    let rounded = mantissa + rounding;
    if rounded & 0x80_0000 != 0 {
        if half_exponent + 1 >= 31 {
            return sign | 0x7c00;
        }
        return sign | (((half_exponent + 1) as u16) << 10);
    }
    sign | ((half_exponent as u16) << 10) | ((rounded >> 13) as u16)
}

#[cfg(target_pointer_width = "32")]
fn f16_to_f32(value: u16) -> f32 {
    let sign = ((value as u32) & 0x8000) << 16;
    let exponent = ((value >> 10) & 0x1f) as i32;
    let mantissa = (value & 0x03ff) as u32;
    let bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut mantissa = mantissa;
            let mut exponent = -14i32;
            while mantissa & 0x0400 == 0 {
                mantissa <<= 1;
                exponent -= 1;
            }
            mantissa &= 0x03ff;
            sign | (((exponent + 127) as u32) << 23) | (mantissa << 13)
        }
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | (((exponent - 15 + 127) as u32) << 23) | (mantissa << 13),
    };
    f32::from_bits(bits)
}

/// Distinguishes compact integer and reduced-precision/general Number values
/// while keeping both inside one numeric family at the language level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumericValue {
    Integer(SmlInt),
    Number(SmlNum),
}

impl NumericValue {
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Integer(value) => value.signed_value() as f64,
            Self::Number(value) => value.to_f64(),
        }
    }
}

impl ScalarCell {
    const fn tagged(tag: Tag, payload: u64) -> Self {
        Self(Cell::new(NAN_BASE | ((tag as u64) << 48) | payload))
    }
    pub const fn null() -> Self {
        Self::tagged(Tag::Null, 0)
    }
    pub const fn undefined() -> Self {
        Self(Cell::new(NAN_BASE | 1))
    }
    pub const fn boolean(value: bool) -> Self {
        Self::tagged(Tag::Bool, value as u64)
    }
    pub fn number(value: f64) -> Self {
        Self(Cell::new(if value.is_nan() {
            NAN_BASE
        } else {
            value.to_bits()
        }))
    }
    pub fn inline_integer(value: i64) -> Option<Self> {
        (MIN_INLINE..=MAX_INLINE)
            .contains(&value)
            .then(|| Self::tagged(Tag::Int, (value as u64) & PAYLOAD))
    }
    pub fn string_literal(id: u64) -> Option<Self> {
        (id < STRING_LITERAL_FLAG).then(|| Self::tagged(Tag::String, STRING_LITERAL_FLAG | id))
    }
    pub fn string_block(handle: usize) -> Result<Self, MemoryError> {
        let encoded = u64::try_from(handle)
            .map_err(|_| MemoryError::Full)?
            .checked_add(1)
            .ok_or(MemoryError::Full)?;
        if encoded >= STRING_LITERAL_FLAG {
            return Err(MemoryError::Full);
        }
        Ok(Self::tagged(Tag::String, encoded))
    }
    pub fn string_literal_id(&self) -> Option<u64> {
        (self.kind() == CellKind::StringLiteral).then(|| self.bits() & (STRING_LITERAL_FLAG - 1))
    }
    fn heap(tag: Tag, index: usize) -> Result<Self, MemoryError> {
        let handle = u64::try_from(index).map_err(|_| MemoryError::Full)?;
        if handle >= PAYLOAD || (tag == Tag::String && handle >= STRING_LITERAL_FLAG) {
            return Err(MemoryError::Full);
        }
        Ok(Self::tagged(tag, handle + 1))
    }
    pub fn bits(&self) -> u64 {
        self.0.get()
    }
    pub fn kind(&self) -> CellKind {
        let bits = self.bits();
        if bits == (NAN_BASE | 1) {
            return CellKind::Undefined;
        }
        if bits & NAN_BASE != NAN_BASE || bits == NAN_BASE {
            return CellKind::Number;
        }
        match (bits & TAG_BITS) >> 48 {
            1 => CellKind::Null,
            2 => CellKind::Boolean,
            3 => CellKind::Integer,
            4 => CellKind::BigInt,
            5 => CellKind::BigNum,
            6 if bits & STRING_LITERAL_FLAG != 0 => CellKind::StringLiteral,
            6 => CellKind::String,
            7 => CellKind::Object,
            _ => CellKind::Number,
        }
    }
    pub fn integer(&self) -> Option<i64> {
        (self.kind() == CellKind::Integer).then(|| (((self.bits() & PAYLOAD) << 16) as i64) >> 16)
    }
    pub fn float(&self) -> Option<f64> {
        (self.kind() == CellKind::Number).then(|| f64::from_bits(self.bits()))
    }
    pub fn handle(&self) -> Option<usize> {
        matches!(
            self.kind(),
            CellKind::BigInt | CellKind::BigNum | CellKind::String | CellKind::Object
        )
        .then(|| {
            let payload = if self.kind() == CellKind::String {
                self.bits() & (STRING_LITERAL_FLAG - 1)
            } else {
                self.bits() & PAYLOAD
            };
            payload as usize - 1
        })
    }
    pub fn schema_type(&self) -> &'static str {
        match self.kind() {
            CellKind::Null => "null",
            CellKind::Undefined => "undefined",
            CellKind::Boolean => "boolean",
            CellKind::Integer | CellKind::BigInt => "integer",
            CellKind::Number | CellKind::BigNum => "number",
            CellKind::String | CellKind::StringLiteral => "string",
            CellKind::Object => "object",
        }
    }
}

/// The arena owns every chunk and preserves references when its Vec grows.
/// Dropping it releases all chunks. Handles must never outlive this arena.
#[derive(Default)]
pub struct ValueArena {
    chunks: Vec<HeapChunk>,
}

impl ValueArena {
    fn push(&mut self, chunk: HeapChunk, tag: Tag) -> Result<ScalarCell, MemoryError> {
        let cell = ScalarCell::heap(tag, self.chunks.len())?;
        self.chunks.try_reserve(1).map_err(|_| MemoryError::Full)?;
        self.chunks.push(chunk);
        Ok(cell)
    }
    pub fn integer(&mut self, value: &BigInt) -> Result<ScalarCell, MemoryError> {
        if let Some(small) = value.to_i64() {
            if let Some(cell) = ScalarCell::inline_integer(small) {
                return Ok(cell);
            }
        }
        let (sign, limbs) = value.to_u64_digits();
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(
                limbs
                    .len()
                    .checked_mul(8)
                    .ok_or(MemoryError::LengthOverflow)?,
            )
            .map_err(|_| MemoryError::Full)?;
        for limb in limbs {
            payload.extend_from_slice(&limb.to_le_bytes());
        }
        self.push(
            HeapChunk::new(HeapKind::BigInt, u8::from(sign == Sign::Minus), 0, &payload)?,
            Tag::BigInt,
        )
    }
    pub fn compact_integer(&mut self, value: &BigInt) -> Result<CompactCell, MemoryError> {
        if let Some(small) = value.to_i32() {
            if let Some(cell) = CompactCell::integer(small) {
                return Ok(cell);
            }
        }
        let (sign, limbs) = value.to_u64_digits();
        let mut payload = Vec::new();
        payload
            .try_reserve_exact(
                limbs
                    .len()
                    .checked_mul(8)
                    .ok_or(MemoryError::LengthOverflow)?,
            )
            .map_err(|_| MemoryError::Full)?;
        for limb in limbs {
            payload.extend_from_slice(&limb.to_le_bytes());
        }
        let cell = CompactCell::heap(5, self.chunks.len())?;
        self.chunks.try_reserve(1).map_err(|_| MemoryError::Full)?;
        self.chunks.push(HeapChunk::new(
            HeapKind::BigInt,
            u8::from(sign == Sign::Minus),
            0,
            &payload,
        )?);
        Ok(cell)
    }
    pub fn compact_number(&mut self, value: f64) -> Result<CompactCell, MemoryError> {
        let cell = CompactCell::heap(6, self.chunks.len())?;
        self.chunks.try_reserve(1).map_err(|_| MemoryError::Full)?;
        self.chunks.push(HeapChunk::new(
            HeapKind::Number64,
            0,
            0,
            &value.to_bits().to_le_bytes(),
        )?);
        Ok(cell)
    }
    pub fn compact_float(&self, cell: &CompactCell) -> Result<f64, MemoryError> {
        if cell.bits() >> 28 != 6 {
            return Err(MemoryError::WrongKind);
        }
        let chunk = self
            .chunks
            .get(cell.handle().ok_or(MemoryError::InvalidHandle)?)
            .ok_or(MemoryError::InvalidHandle)?;
        if chunk.kind() != HeapKind::Number64 || chunk.payload().len() != 8 {
            return Err(MemoryError::WrongKind);
        }
        Ok(f64::from_bits(u64::from_le_bytes(
            chunk.payload().try_into().expect("eight bytes"),
        )))
    }
    pub fn compact_bigint(&self, cell: &CompactCell) -> Result<BigInt, MemoryError> {
        if let Some(small) = cell.inline_integer() {
            return Ok(BigInt::from(small));
        }
        if cell.bits() >> 28 != 5 {
            return Err(MemoryError::WrongKind);
        }
        let chunk = self
            .chunks
            .get(cell.handle().ok_or(MemoryError::InvalidHandle)?)
            .ok_or(MemoryError::InvalidHandle)?;
        if chunk.kind() != HeapKind::BigInt {
            return Err(MemoryError::WrongKind);
        }
        Ok(BigInt::from_bytes_le(
            if chunk.flags() & 1 != 0 {
                Sign::Minus
            } else {
                Sign::Plus
            },
            chunk.payload(),
        ))
    }
    pub fn big_num(
        &mut self,
        format: BigNumFormat,
        bytes: &[u8],
    ) -> Result<ScalarCell, MemoryError> {
        let expected = match format {
            BigNumFormat::Float80 => Some(10),
            BigNumFormat::Float128 | BigNumFormat::Decimal128 | BigNumFormat::DoubleDouble => {
                Some(16)
            }
            BigNumFormat::Custom => None,
        };
        if expected.is_some_and(|n| n != bytes.len()) {
            return Err(MemoryError::InvalidPayload);
        }
        self.push(
            HeapChunk::new(HeapKind::BigNum, 0, format as u16, bytes)?,
            Tag::BigNum,
        )
    }
    pub fn string(&mut self, value: &str) -> Result<ScalarCell, MemoryError> {
        self.push(
            HeapChunk::new(HeapKind::String, 0, 0, value.as_bytes())?,
            Tag::String,
        )
    }
    pub fn object(&mut self, slots: &[ScalarCell]) -> Result<ScalarCell, MemoryError> {
        self.object_with_prototype(None, slots)
    }
    pub fn object_with_prototype(
        &mut self,
        prototype: Option<&ScalarCell>,
        slots: &[ScalarCell],
    ) -> Result<ScalarCell, MemoryError> {
        let mut bytes = Vec::new();
        let size = slots
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(8))
            .ok_or(MemoryError::LengthOverflow)?;
        bytes
            .try_reserve_exact(size)
            .map_err(|_| MemoryError::Full)?;
        bytes.extend_from_slice(&prototype.map_or(0, ScalarCell::bits).to_le_bytes());
        for slot in slots {
            bytes.extend_from_slice(&slot.bits().to_le_bytes());
        }
        self.push(HeapChunk::new(HeapKind::Object, 0, 0, &bytes)?, Tag::Object)
    }
    pub fn chunk(&self, cell: &ScalarCell) -> Result<&HeapChunk, MemoryError> {
        let index = cell.handle().ok_or(MemoryError::InvalidHandle)?;
        let chunk = self.chunks.get(index).ok_or(MemoryError::InvalidHandle)?;
        let valid = matches!(
            (cell.kind(), chunk.kind()),
            (CellKind::BigInt, HeapKind::BigInt)
                | (CellKind::BigNum, HeapKind::BigNum)
                | (CellKind::String, HeapKind::String)
                | (CellKind::Object, HeapKind::Object)
        );
        if !valid {
            return Err(MemoryError::WrongKind);
        }
        Ok(chunk)
    }
    pub fn object_slot(
        &self,
        object: &ScalarCell,
        index: usize,
    ) -> Result<ScalarCell, MemoryError> {
        let chunk = self.chunk(object)?;
        if chunk.kind() != HeapKind::Object {
            return Err(MemoryError::WrongKind);
        }
        let start = index
            .checked_add(1)
            .and_then(|n| n.checked_mul(8))
            .ok_or(MemoryError::InvalidHandle)?;
        let end = start.checked_add(8).ok_or(MemoryError::InvalidHandle)?;
        let bytes = chunk
            .payload()
            .get(start..end)
            .ok_or(MemoryError::InvalidHandle)?;
        Ok(ScalarCell(Cell::new(u64::from_le_bytes(
            bytes.try_into().expect("eight bytes"),
        ))))
    }
    pub fn set_object_slot(
        &mut self,
        object: &ScalarCell,
        index: usize,
        value: &ScalarCell,
    ) -> Result<(), MemoryError> {
        let handle = object.handle().ok_or(MemoryError::InvalidHandle)?;
        let chunk = self
            .chunks
            .get_mut(handle)
            .ok_or(MemoryError::InvalidHandle)?;
        if object.kind() != CellKind::Object || chunk.kind() != HeapKind::Object {
            return Err(MemoryError::WrongKind);
        }
        let start = index
            .checked_add(2)
            .and_then(|n| n.checked_mul(8))
            .ok_or(MemoryError::InvalidHandle)?;
        let dst = chunk
            .bytes
            .get_mut(start..start + 8)
            .ok_or(MemoryError::InvalidHandle)?;
        dst.copy_from_slice(&value.bits().to_le_bytes());
        Ok(())
    }
    pub fn bigint(&self, cell: &ScalarCell) -> Result<BigInt, MemoryError> {
        if let Some(value) = cell.integer() {
            return Ok(BigInt::from(value));
        }
        let chunk = self.chunk(cell)?;
        if chunk.kind() != HeapKind::BigInt || chunk.payload().len() % 8 != 0 {
            return Err(MemoryError::WrongKind);
        }
        Ok(BigInt::from_bytes_le(
            if chunk.flags() & 1 != 0 {
                Sign::Minus
            } else {
                Sign::Plus
            },
            chunk.payload(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inline_and_spilled_integers_round_trip() {
        assert_eq!(std::mem::size_of::<ScalarCell>(), 8);
        assert_eq!(std::mem::size_of::<HeapHeader>(), 8);
        let mut arena = ValueArena::default();
        for n in [-(1i64 << 47), (1i64 << 47) - 1, 1i64 << 47] {
            let cell = arena.integer(&BigInt::from(n)).unwrap();
            assert_eq!(arena.bigint(&cell).unwrap(), BigInt::from(n));
        }
        let huge = (BigInt::from(1u8) << 4096u32) + 7;
        let cell = arena.integer(&huge).unwrap();
        assert_eq!(cell.schema_type(), "integer");
        assert_eq!(arena.bigint(&cell).unwrap(), huge);
    }
    #[test]
    fn nan_zero_and_chunks() {
        assert!(ScalarCell::number(f64::NAN).float().unwrap().is_nan());
        assert!(ScalarCell::number(-0.0).float().unwrap().is_sign_negative());
        assert_eq!(ScalarCell::undefined().kind(), CellKind::Undefined);
        let mut arena = ValueArena::default();
        let text = arena.string("hello").unwrap();
        assert_eq!(arena.chunk(&text).unwrap().payload(), b"hello");
        let number = arena.big_num(BigNumFormat::Float128, &[0; 16]).unwrap();
        assert_eq!(arena.chunk(&number).unwrap().payload().len(), 16);
        let object = arena
            .object(&[ScalarCell::null(), ScalarCell::boolean(false)])
            .unwrap();
        arena
            .set_object_slot(&object, 1, &ScalarCell::boolean(true))
            .unwrap();
        assert_eq!(
            arena.object_slot(&object, 1).unwrap().schema_type(),
            "boolean"
        );
    }
    #[test]
    fn compact_cells_preserve_f64_and_large_integer_on_32_bit_layout() {
        assert_eq!(std::mem::size_of::<CompactCell>(), 4);
        let mut arena = ValueArena::default();
        let number = arena.compact_number(-0.0).unwrap();
        assert!(arena.compact_float(&number).unwrap().is_sign_negative());
        let huge = BigInt::from(1u8) << 512u32;
        let integer = arena.compact_integer(&huge).unwrap();
        assert_eq!(arena.compact_bigint(&integer).unwrap(), huge);
        assert_eq!(
            CompactCell::integer(-17).unwrap().inline_integer(),
            Some(-17)
        );
        assert!(SmlInt::new(SML_INT_MAX).is_ok());
        assert!(SmlInt::new_unsigned((1u64 << SML_INT_BITS) - 1).is_ok());
        assert_eq!(
            SmlInt::new_unsigned(1u64 << SML_INT_BITS),
            Err(MemoryError::InvalidPayload)
        );
        #[cfg(target_pointer_width = "32")]
        assert_eq!(
            SmlInt::new(SML_INT_MAX + 1),
            Err(MemoryError::InvalidPayload)
        );
        let small = SmlInt::new(42).unwrap();
        assert_eq!(std::mem::size_of::<SmlInt>(), 4);
        assert_eq!(
            SML_INT_BITS,
            if cfg!(target_pointer_width = "32") {
                16
            } else {
                32
            }
        );
        assert_eq!(
            SML_NUM_BITS,
            if cfg!(target_pointer_width = "32") {
                16
            } else {
                32
            }
        );
        assert_eq!(
            NORMAL_CELL_BITS,
            if cfg!(target_pointer_width = "32") {
                32
            } else {
                64
            }
        );
        assert_eq!(NumericValue::Integer(small).as_f64(), 42.0);
        let sml_num = SmlNum::from_f64(2.5).unwrap();
        assert_eq!(NumericValue::Number(sml_num).as_f64(), 2.5);
        assert_eq!(sml_num.runtime_cell().schema_type(), "number");
        let negative_zero = SmlNum::from_f64(-0.0).unwrap();
        assert!(negative_zero.to_f64().is_sign_negative());
        assert_eq!(
            SmlInt::new(SML_INT_MIN)
                .unwrap()
                .runtime_cell(false)
                .schema_type(),
            "integer"
        );
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(small.runtime_cell(false).bits() >> 28, 4);
            assert_eq!(
                SmlInt::new_unsigned(42).unwrap().runtime_cell(true).bits() >> 28,
                11
            );
            assert_eq!(sml_num.runtime_cell().bits() >> 28, 10);
        }
    }
}
