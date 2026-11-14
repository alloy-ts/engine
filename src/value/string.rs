use std::collections::HashMap;

use crate::value::memory::{HeapKind, MemoryError};
#[cfg(target_pointer_width = "32")]
use crate::value::memory::CompactCell;
#[cfg(not(target_pointer_width = "32"))]
use crate::value::memory::ScalarCell;

const STRING_HEADER_LEN: usize = 16;
const STRING_HASH_FLAG: u8 = 1;
const MAX_LITERAL_CODE_UNITS: usize = 32;
#[cfg(target_pointer_width = "32")]
const MAX_STRING_BLOCKS: usize = (1usize << 27) - 1;
#[cfg(not(target_pointer_width = "32"))]
const MAX_STRING_BLOCKS: usize = u32::MAX as usize;

#[cfg(target_pointer_width = "32")]
pub type StringLiteralId = u16;
#[cfg(not(target_pointer_width = "32"))]
pub type StringLiteralId = u32;

pub type StringBlockId = u32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum StringEncoding {
  Ascii = 1,
  Utf8 = 2,
  Utf16Le = 3,
}

impl TryFrom<u8> for StringEncoding {
  type Error = MemoryError;

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(Self::Ascii),
      2 => Ok(Self::Utf8),
      3 => Ok(Self::Utf16Le),
      _ => Err(MemoryError::InvalidPayload),
    }
  }
}

/// Logical view of the 16-byte prefix stored at the beginning of each string
/// allocation. The prefix and encoded text bytes share one contiguous block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct StringBlockHeader {
  pub kind: u8,
  pub encoding: StringEncoding,
  pub flags: u8,
  pub reserved: u8,
  pub byte_len: u32,
  pub code_unit_len: u32,
  pub hash: u32,
}

/// Immutable contiguous string storage. UTF-16 payloads are little-endian and
/// may contain unpaired surrogates, as JavaScript strings can.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringBlock {
  bytes: Box<[u8]>,
}

impl StringBlock {
  pub fn from_utf8(value: &str) -> Result<Self, MemoryError> {
    let code_unit_len = u32::try_from(value.encode_utf16().count())
      .map_err(|_| MemoryError::LengthOverflow)?;
    let encoding = if value.is_ascii() { StringEncoding::Ascii } else { StringEncoding::Utf8 };
    Self::build(encoding, code_unit_len, value.as_bytes())
  }

  pub fn from_utf16(value: &[u16]) -> Result<Self, MemoryError> {
    let code_unit_len = u32::try_from(value.len()).map_err(|_| MemoryError::LengthOverflow)?;
    if value.iter().all(|unit| *unit <= 0x7f) {
      let mut bytes = Vec::new();
      bytes.try_reserve_exact(value.len()).map_err(|_| MemoryError::Full)?;
      bytes.extend(value.iter().map(|unit| *unit as u8));
      return Self::build(StringEncoding::Ascii, code_unit_len, &bytes);
    }

    let byte_len = value.len().checked_mul(2).ok_or(MemoryError::LengthOverflow)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(byte_len).map_err(|_| MemoryError::Full)?;
    for unit in value {
      bytes.extend_from_slice(&unit.to_le_bytes());
    }
    Self::build(StringEncoding::Utf16Le, code_unit_len, &bytes)
  }

  fn build(
    encoding: StringEncoding,
    code_unit_len: u32,
    payload: &[u8],
  ) -> Result<Self, MemoryError> {
    let byte_len = u32::try_from(payload.len()).map_err(|_| MemoryError::LengthOverflow)?;
    let units = match encoding {
      StringEncoding::Ascii => payload.iter().map(|byte| u16::from(*byte)).collect::<Vec<_>>(),
      StringEncoding::Utf8 => std::str::from_utf8(payload)
        .map_err(|_| MemoryError::InvalidPayload)?
        .encode_utf16()
        .collect::<Vec<_>>(),
      StringEncoding::Utf16Le => {
        if payload.len() % 2 != 0 { return Err(MemoryError::InvalidPayload); }
        payload
          .chunks_exact(2)
          .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
          .collect::<Vec<_>>()
      }
    };
    if units.len() != code_unit_len as usize { return Err(MemoryError::InvalidPayload); }
    let hash = hash_utf16(&units);
    let capacity = STRING_HEADER_LEN.checked_add(payload.len()).ok_or(MemoryError::LengthOverflow)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(capacity).map_err(|_| MemoryError::Full)?;
    bytes.extend_from_slice(&[
      HeapKind::String as u8,
      encoding as u8,
      STRING_HASH_FLAG,
      0,
    ]);
    bytes.extend_from_slice(&byte_len.to_le_bytes());
    bytes.extend_from_slice(&code_unit_len.to_le_bytes());
    bytes.extend_from_slice(&hash.to_le_bytes());
    bytes.extend_from_slice(payload);
    Ok(Self { bytes: bytes.into_boxed_slice() })
  }

  pub fn header(&self) -> Result<StringBlockHeader, MemoryError> {
    if self.bytes.len() < STRING_HEADER_LEN { return Err(MemoryError::InvalidPayload); }
    if self.bytes[0] != HeapKind::String as u8 { return Err(MemoryError::WrongKind); }
    let encoding = StringEncoding::try_from(self.bytes[1])?;
    let byte_len = u32::from_le_bytes(self.bytes[4..8].try_into().map_err(|_| MemoryError::InvalidPayload)?);
    let code_unit_len = u32::from_le_bytes(self.bytes[8..12].try_into().map_err(|_| MemoryError::InvalidPayload)?);
    let hash = u32::from_le_bytes(self.bytes[12..16].try_into().map_err(|_| MemoryError::InvalidPayload)?);
    let expected_len = STRING_HEADER_LEN
      .checked_add(byte_len as usize)
      .ok_or(MemoryError::LengthOverflow)?;
    if self.bytes.len() != expected_len {
      return Err(MemoryError::InvalidPayload);
    }
    Ok(StringBlockHeader {
      kind: self.bytes[0],
      encoding,
      flags: self.bytes[2],
      reserved: self.bytes[3],
      byte_len,
      code_unit_len,
      hash,
    })
  }

  pub fn encoding(&self) -> Result<StringEncoding, MemoryError> { Ok(self.header()?.encoding) }
  pub fn byte_len(&self) -> Result<u32, MemoryError> { Ok(self.header()?.byte_len) }
  pub fn code_unit_len(&self) -> Result<u32, MemoryError> { Ok(self.header()?.code_unit_len) }
  pub fn hash(&self) -> Result<u32, MemoryError> { Ok(self.header()?.hash) }
  pub fn allocation(&self) -> &[u8] { &self.bytes }
  pub fn encoded_bytes(&self) -> Result<&[u8], MemoryError> {
    self.header()?;
    Ok(&self.bytes[STRING_HEADER_LEN..])
  }

  /// Borrows bytes directly when the block is already UTF-8 or ASCII.
  pub fn as_utf8(&self) -> Result<Option<&str>, MemoryError> {
    match self.encoding()? {
      StringEncoding::Ascii | StringEncoding::Utf8 => std::str::from_utf8(self.encoded_bytes()?)
        .map(Some)
        .map_err(|_| MemoryError::InvalidPayload),
      StringEncoding::Utf16Le => Ok(None),
    }
  }

  pub fn to_utf16(&self) -> Result<Vec<u16>, MemoryError> {
    match self.encoding()? {
      StringEncoding::Ascii => Ok(self.encoded_bytes()?.iter().map(|byte| u16::from(*byte)).collect()),
      StringEncoding::Utf8 => Ok(std::str::from_utf8(self.encoded_bytes()?)
        .map_err(|_| MemoryError::InvalidPayload)?
        .encode_utf16()
        .collect()),
      StringEncoding::Utf16Le => {
        let bytes = self.encoded_bytes()?;
        if bytes.len() % 2 != 0 { return Err(MemoryError::InvalidPayload); }
        Ok(bytes.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect())
      }
    }
  }

  pub fn to_utf8_lossy(&self) -> Result<String, MemoryError> {
    if let Some(value) = self.as_utf8()? { return Ok(value.to_owned()); }
    Ok(String::from_utf16_lossy(&self.to_utf16()?))
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StringValue {
  Literal(StringLiteralId),
  Block(StringBlockId),
}

impl StringValue {
  #[cfg(target_pointer_width = "32")]
  pub fn runtime_cell(self) -> Option<CompactCell> {
    match self {
      Self::Literal(id) => Some(CompactCell::string_literal(id)),
      Self::Block(id) => CompactCell::string_block(id),
    }
  }

  #[cfg(not(target_pointer_width = "32"))]
  pub fn runtime_cell(self) -> Option<ScalarCell> {
    match self {
      Self::Literal(id) => ScalarCell::string_literal(id as u64),
      Self::Block(id) => ScalarCell::string_block(id as usize).ok(),
    }
  }
}

/// Per-runtime literal registry and owner for materialized string blocks.
/// Literal IDs are compact handles; string-producing operations return blocks.
#[derive(Default)]
pub struct StringTable {
  literal_index: HashMap<Vec<u16>, StringLiteralId>,
  literal_blocks: Vec<StringBlockId>,
  blocks: Vec<StringBlock>,
}

impl StringTable {
  pub fn literal_utf8(&mut self, value: &str) -> Result<StringValue, MemoryError> {
    let units = value.encode_utf16().collect::<Vec<_>>();
    if units.len() <= MAX_LITERAL_CODE_UNITS {
      return self.intern_literal_utf16(&units);
    }
    self.block_utf8(value)
  }

  pub fn literal_utf16(&mut self, value: &[u16]) -> Result<StringValue, MemoryError> {
    if value.len() <= MAX_LITERAL_CODE_UNITS {
      return self.intern_literal_utf16(value);
    }
    self.block_utf16(value)
  }

  /// Explicitly creates a contiguous encoded block, even for short text.
  pub fn block_utf8(&mut self, value: &str) -> Result<StringValue, MemoryError> {
    self.store_block(StringBlock::from_utf8(value)?)
  }

  /// Explicitly creates a contiguous UTF-16LE block, preserving lone surrogates.
  pub fn block_utf16(&mut self, value: &[u16]) -> Result<StringValue, MemoryError> {
    self.store_block(StringBlock::from_utf16(value)?)
  }

  pub fn resolve(&self, value: StringValue) -> Result<&StringBlock, MemoryError> {
    let block_id = match value {
      StringValue::Literal(id) => *self.literal_blocks.get(id as usize).ok_or(MemoryError::InvalidHandle)?,
      StringValue::Block(id) => id,
    };
    self.blocks.get(block_id as usize).ok_or(MemoryError::InvalidHandle)
  }

  /// String transformations materialize their output as a block.
  pub fn materialize(&mut self, value: StringValue) -> Result<StringValue, MemoryError> {
    if matches!(value, StringValue::Block(_)) { return Ok(value); }
    let units = self.resolve(value)?.to_utf16()?;
    self.block_utf16(&units)
  }

  pub fn concat(&mut self, left: StringValue, right: StringValue) -> Result<StringValue, MemoryError> {
    let mut units = self.resolve(left)?.to_utf16()?;
    let right_units = self.resolve(right)?.to_utf16()?;
    units.try_reserve(right_units.len()).map_err(|_| MemoryError::Full)?;
    units.extend_from_slice(&right_units);
    self.block_utf16(&units)
  }

  /// JavaScript-style `slice` indices are interpreted as UTF-16 code units.
  pub fn slice(
    &mut self,
    value: StringValue,
    start: i64,
    end: Option<i64>,
  ) -> Result<StringValue, MemoryError> {
    let mut units = self.resolve(value)?.to_utf16()?;
    let length = units.len();
    let start = relative_index(start, length);
    let end = end.map_or(length, |index| relative_index(index, length));
    if start >= end {
      units.clear();
    } else {
      units = units[start..end].to_vec();
    }
    self.block_utf16(&units)
  }

  pub fn equals(&self, left: StringValue, right: StringValue) -> Result<bool, MemoryError> {
    if left == right { return Ok(true); }
    Ok(self.resolve(left)?.to_utf16()? == self.resolve(right)?.to_utf16()?)
  }

  fn intern_literal_utf16(&mut self, value: &[u16]) -> Result<StringValue, MemoryError> {
    if let Some(id) = self.literal_index.get(value) { return Ok(StringValue::Literal(*id)); }

    let literal_id = StringLiteralId::try_from(self.literal_blocks.len());
    let Some(literal_id) = literal_id.ok() else { return self.block_utf16(value); };
    self.literal_index.try_reserve(1).map_err(|_| MemoryError::Full)?;
    self.literal_blocks.try_reserve(1).map_err(|_| MemoryError::Full)?;
    let mut key = Vec::new();
    key.try_reserve_exact(value.len()).map_err(|_| MemoryError::Full)?;
    key.extend_from_slice(value);
    let block = self.store_block_inner(StringBlock::from_utf16(value)?)?;
    let block_id = match block { StringValue::Block(id) => id, StringValue::Literal(_) => unreachable!() };
    self.literal_blocks.push(block_id);
    self.literal_index.insert(key, literal_id);
    Ok(StringValue::Literal(literal_id))
  }

  fn store_block(&mut self, block: StringBlock) -> Result<StringValue, MemoryError> {
    self.store_block_inner(block)
  }

  fn store_block_inner(&mut self, block: StringBlock) -> Result<StringValue, MemoryError> {
    let id = u32::try_from(self.blocks.len()).map_err(|_| MemoryError::Full)?;
    if self.blocks.len() >= MAX_STRING_BLOCKS { return Err(MemoryError::Full); }
    self.blocks.try_reserve(1).map_err(|_| MemoryError::Full)?;
    self.blocks.push(block);
    Ok(StringValue::Block(id))
  }
}

fn relative_index(index: i64, length: usize) -> usize {
  let length = i64::try_from(length).unwrap_or(i64::MAX);
  if index < 0 { length.saturating_add(index).max(0) as usize }
  else { index.min(length).max(0) as usize }
}

fn hash_utf16(units: &[u16]) -> u32 {
  let mut hash = 0x811c_9dc5u32;
  for unit in units {
    for byte in unit.to_le_bytes() {
      hash ^= u32::from(byte);
      hash = hash.wrapping_mul(0x0100_0193);
    }
  }
  hash
}

#[cfg(test)]
mod tests {
  use super::*;
  #[cfg(not(target_pointer_width = "32"))]
  use crate::value::memory::CellKind;

  #[test]
  fn utf8_and_utf16_blocks_share_code_unit_hashes() {
    let utf8 = StringBlock::from_utf8("A🦀").unwrap();
    let utf16 = StringBlock::from_utf16(&[b'A' as u16, 0xd83e, 0xdd80]).unwrap();
    assert_eq!(utf8.encoding().unwrap(), StringEncoding::Utf8);
    assert_eq!(utf16.encoding().unwrap(), StringEncoding::Utf16Le);
    assert_eq!(utf8.code_unit_len().unwrap(), 3);
    assert_eq!(utf8.hash().unwrap(), utf16.hash().unwrap());
    assert_eq!(utf16.to_utf16().unwrap(), [b'A' as u16, 0xd83e, 0xdd80]);
    assert_eq!(utf8.allocation().len(), 16 + utf8.byte_len().unwrap() as usize);
  }

  #[test]
  fn literals_are_deduplicated_and_transformations_materialize_blocks() {
    let mut table = StringTable::default();
    let literal_a = table.literal_utf8("ready").unwrap();
    let literal_b = table.literal_utf16(&[114, 101, 97, 100, 121]).unwrap();
    assert_eq!(literal_a, literal_b);
    assert!(matches!(literal_a, StringValue::Literal(_)));
    assert_eq!(table.resolve(literal_a).unwrap().to_utf8_lossy().unwrap(), "ready");

    let suffix = table.block_utf8("!").unwrap();
    let joined = table.concat(literal_a, suffix).unwrap();
    assert!(matches!(joined, StringValue::Block(_)));
    let sliced = table.slice(joined, 1, Some(-1)).unwrap();
    assert!(matches!(sliced, StringValue::Block(_)));
    assert_eq!(table.resolve(sliced).unwrap().to_utf8_lossy().unwrap(), "eady");
  }

  #[test]
  fn lone_surrogates_survive_utf16_storage_and_code_unit_slicing() {
    let mut table = StringTable::default();
    let value = table.block_utf16(&[0xd800, b'x' as u16]).unwrap();
    assert_eq!(table.resolve(value).unwrap().to_utf16().unwrap(), [0xd800, b'x' as u16]);
    assert_eq!(table.resolve(value).unwrap().to_utf8_lossy().unwrap(), "�x");
    let literal = table.literal_utf16(&[0xd800]).unwrap();
    let sliced = table.slice(value, 0, Some(1)).unwrap();
    assert!(table.equals(literal, sliced).unwrap());
  }

  #[test]
  fn string_cells_distinguish_literals_from_blocks() {
    let mut table = StringTable::default();
    let literal = table.literal_utf8("key").unwrap();
    let block = table.block_utf8("key").unwrap();
    #[cfg(target_pointer_width = "32")]
    {
      assert_eq!(literal.runtime_cell().unwrap().schema_type(), "string");
      assert_eq!(block.runtime_cell().unwrap().schema_type(), "string");
      assert!(literal.runtime_cell().unwrap().string_literal_id().is_some());
      assert!(block.runtime_cell().unwrap().string_block_handle().is_some());
    }
    #[cfg(not(target_pointer_width = "32"))]
    {
      assert_eq!(literal.runtime_cell().unwrap().kind(), CellKind::StringLiteral);
      assert_eq!(block.runtime_cell().unwrap().kind(), CellKind::String);
    }
  }
}
