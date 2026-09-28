//! Behavior-preserving translations of deterministic helpers in `src/util.c`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use crate::c_types::MemBlk;

/// Growable byte buffer corresponding to the C `ByteArray`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ByteArray {
    data: Vec<u8>,
    logical_capacity: usize,
}

impl ByteArray {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The capacity selected by the C growth rule. The allocator may reserve
    /// more storage internally, which is intentionally not observable here.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.logical_capacity
    }

    pub fn resize(&mut self, new_size: usize) {
        if new_size > self.logical_capacity {
            let minimum = self
                .logical_capacity
                .wrapping_add(self.logical_capacity >> 1)
                .wrapping_add(8);
            self.logical_capacity = new_size.max(minimum);
            self.data
                .reserve(self.logical_capacity.saturating_sub(self.data.len()));
        }
        // C leaves newly allocated bytes uninitialized. Rust cannot expose
        // uninitialized bytes safely; all C callers overwrite them before a
        // read, so zero filling preserves every observable use.
        self.data.resize(new_size, 0);
    }

    pub fn destroy(&mut self) {
        self.data = Vec::new();
        self.logical_capacity = 0;
    }

    pub fn append_data(&mut self, data: &[u8]) {
        let old_size = self.len();
        self.resize(old_size.wrapping_add(data.len()));
        self.data[old_size..].copy_from_slice(data);
    }

    pub fn append_byte(&mut self, value: u8) {
        let old_size = self.len();
        self.resize(old_size.wrapping_add(1));
        self.data[old_size] = value;
    }
}

/// Reads a complete file and appends the C helper's extra NUL terminator.
/// The returned length excludes that terminator.
pub fn read_whole_file(path: impl AsRef<Path>) -> io::Result<(Vec<u8>, usize)> {
    let mut data = fs::read(path)?;
    let length = data.len();
    data.push(0);
    Ok((data, length))
}

/// C's byte-wise ASCII `ToLower`; bytes outside `A..=Z` are unchanged.
#[must_use]
pub const fn to_lower(value: u8) -> u8 {
    if value >= b'A' && value <= b'Z' {
        value + 32
    } else {
        value
    }
}

/// Ownership-safe equivalent of replacing a heap-allocated C string.
pub fn str_set(destination: &mut String, value: &str) {
    destination.clear();
    destination.push_str(value);
}

/// Ownership-safe equivalent of `StrFmt`. Callers use `format_args!` to retain
/// Rust's checked formatting while preserving the C helper's owned result.
#[must_use]
pub fn str_fmt(arguments: fmt::Arguments<'_>) -> String {
    fmt::format(arguments)
}

#[must_use]
pub fn string_equals_no_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Returns the remainder of `a` after a case-insensitive ASCII prefix.
#[must_use]
pub fn string_starts_with_no_case<'a>(a: &'a str, prefix: &str) -> Option<&'a str> {
    let prefix_len = prefix.len();
    let head = a.get(..prefix_len)?;
    head.eq_ignore_ascii_case(prefix).then(|| &a[prefix_len..])
}

/// C's `NextDelim`, represented without writing NUL bytes into the input.
pub fn next_delim<'a>(input: &mut Option<&'a str>, separator: char) -> Option<&'a str> {
    let text = input.take()?;
    let text = text.trim_start_matches([' ', '\t']);
    if let Some(offset) = text.find(separator) {
        let after = offset + separator.len_utf8();
        *input = Some(&text[after..]);
        Some(&text[..offset])
    } else {
        *input = None;
        Some(text)
    }
}

/// C's line parser: `#` starts a comment and only space, tab, and carriage
/// return count as removable whitespace.
pub fn next_line_strip_comments<'a>(input: &mut Option<&'a str>) -> Option<&'a str> {
    let text = input.take()?;
    let (line, remainder) = match text.find('\n') {
        Some(offset) => (&text[..offset], Some(&text[offset + 1..])),
        None => (text, None),
    };
    *input = remainder;
    let line = line.split_once('#').map_or(line, |(before, _)| before);
    Some(
        line.trim_end_matches(['\r', ' ', '\t'])
            .trim_start_matches([' ', '\t']),
    )
}

/// Returns the next space-separated token, honoring the C helper's simple
/// double-quote convention, and advances `input` to the remaining text.
pub fn next_possibly_quoted_string<'a>(input: &mut &'a str) -> &'a str {
    let text = input.trim_start_matches([' ', '\t']);
    let (token, remainder) = if let Some(quoted) = text.strip_prefix('"') {
        match quoted.find('"') {
            Some(offset) => (&quoted[..offset], &quoted[offset + 1..]),
            None => (quoted, ""),
        }
    } else {
        match text.find([' ', '\t']) {
            Some(offset) => (&text[..offset], &text[offset + 1..]),
            None => (text, ""),
        }
    };
    *input = remainder.trim_start_matches([' ', '\t']);
    token
}

#[must_use]
pub fn replace_filename_with_new_path(old_path: &str, new_path: &str) -> String {
    match old_path.rfind(['/', '\\']) {
        Some(offset) => format!("{}{}", &old_path[..=offset], new_path),
        None => new_path.to_owned(),
    }
}

#[must_use]
pub fn split_key_value(text: &str) -> Option<(&str, &str)> {
    let (key, value) = text.split_once('=')?;
    Some((
        key.trim_end_matches([' ', '\t']),
        value.trim_start_matches([' ', '\t']),
    ))
}

#[must_use]
pub fn skip_prefix<'a>(big: &'a str, little: &str) -> Option<&'a str> {
    big.strip_prefix(little)
}

/// Decodes one entry from the compact indexed-block representation used by
/// assets and dialogue. Integer table entries are explicitly little-endian.
#[must_use]
pub fn find_index_in_memblk<'a>(data: MemBlk<'a>, index: usize) -> Option<MemBlk<'a>> {
    let bytes = data.bytes;
    if bytes.len() < 2 {
        return None;
    }

    let end = bytes.len() - 2;
    let marker = read_u16_le(bytes, end)? as usize;
    let (count, entry_width) = if marker < 8192 {
        (marker, 2)
    } else {
        (marker - 8192, 4)
    };

    let table_size = count.checked_mul(entry_width)?;
    if index > count || table_size > end {
        return None;
    }

    let left = if index == 0 {
        table_size
    } else {
        table_size.checked_add(read_offset(bytes, (index - 1) * entry_width, entry_width)?)?
    };
    let right = if index == count {
        end
    } else {
        table_size.checked_add(read_offset(bytes, index * entry_width, entry_width)?)?
    };

    if left > right || right > end {
        return None;
    }
    Some(MemBlk::new(&bytes[left..right]))
}

fn read_offset(bytes: &[u8], offset: usize, width: usize) -> Option<usize> {
    match width {
        2 => Some(read_u16_le(bytes, offset)? as usize),
        4 => Some(read_u32_le(bytes, offset)? as usize),
        _ => None,
    }
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn bps_decode_int(input: &[u8], cursor: &mut usize, end: usize) -> Option<u64> {
    let mut data = 0_u64;
    let mut shift = 1_u64;
    loop {
        if *cursor >= end {
            return None;
        }
        let value = input[*cursor];
        *cursor += 1;
        data = data.checked_add(u64::from(value & 0x7f).checked_mul(shift)?)?;
        if value & 0x80 != 0 {
            return Some(data);
        }
        shift = shift.checked_shl(7)?;
        data = data.checked_add(shift)?;
    }
}

const CRC32_POLYNOMIAL: u32 = 0xedb8_8320;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ ((crc & 1).wrapping_mul(CRC32_POLYNOMIAL));
        }
    }
    crc ^ u32::MAX
}

/// Applies a BPS patch with the same four command modes as the C reference.
/// Invalid patches return `None` instead of invoking C undefined behavior.
#[must_use]
pub fn apply_bps(source: &[u8], patch: &[u8]) -> Option<Vec<u8>> {
    if patch.len() < 16 || patch.get(..4)? != b"BPS1" {
        return None;
    }
    let commands_end = patch.len() - 12;
    if crc32(source) != read_u32_le(patch, commands_end)?
        || crc32(&patch[..patch.len() - 4]) != read_u32_le(patch, commands_end + 8)?
    {
        return None;
    }

    let mut cursor = 4;
    let source_size = usize::try_from(bps_decode_int(patch, &mut cursor, commands_end)?).ok()?;
    let target_size = usize::try_from(bps_decode_int(patch, &mut cursor, commands_end)?).ok()?;
    let _metadata_size = bps_decode_int(patch, &mut cursor, commands_end)?;
    if source_size != source.len() {
        return None;
    }

    let mut output = Vec::with_capacity(target_size);
    let mut source_relative = 0_i64;
    let mut target_relative = 0_i64;

    while cursor < commands_end {
        let command = bps_decode_int(patch, &mut cursor, commands_end)?;
        let length = usize::try_from((command >> 2).checked_add(1)?).ok()?;
        match command & 3 {
            0 => {
                let start = output.len();
                let end = start.checked_add(length)?;
                output.extend_from_slice(source.get(start..end)?);
            }
            1 => {
                let end = cursor.checked_add(length)?;
                output.extend_from_slice(patch.get(cursor..end)?);
                cursor = end;
            }
            2 => {
                let relative = bps_decode_int(patch, &mut cursor, commands_end)?;
                source_relative = apply_relative_offset(source_relative, relative)?;
                for _ in 0..length {
                    let position = usize::try_from(source_relative).ok()?;
                    output.push(*source.get(position)?);
                    source_relative = source_relative.checked_add(1)?;
                }
            }
            3 => {
                let relative = bps_decode_int(patch, &mut cursor, commands_end)?;
                target_relative = apply_relative_offset(target_relative, relative)?;
                for _ in 0..length {
                    let position = usize::try_from(target_relative).ok()?;
                    // Copying one byte at a time deliberately permits overlap.
                    output.push(*output.get(position)?);
                    target_relative = target_relative.checked_add(1)?;
                }
            }
            _ => unreachable!(),
        }
        if output.len() > target_size {
            return None;
        }
    }

    if output.len() != target_size || crc32(&output) != read_u32_le(patch, commands_end + 4)? {
        return None;
    }
    Some(output)
}

fn apply_relative_offset(current: i64, encoded: u64) -> Option<i64> {
    let magnitude = i64::try_from(encoded >> 1).ok()?;
    if encoded & 1 != 0 {
        current.checked_sub(magnitude)
    } else {
        current.checked_add(magnitude)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_array_uses_reference_growth_rule() {
        let mut bytes = ByteArray::default();
        bytes.append_byte(1);
        assert_eq!(bytes.capacity(), 8);
        bytes.append_data(&[2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(bytes.capacity(), 20);
        assert_eq!(bytes.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        bytes.destroy();
        assert!(bytes.is_empty());
        assert_eq!(bytes.capacity(), 0);
    }

    #[test]
    fn parsers_match_c_whitespace_and_comment_rules() {
        let mut fields = Some("\t alpha , beta");
        assert_eq!(next_delim(&mut fields, ','), Some("alpha "));
        assert_eq!(next_delim(&mut fields, ','), Some("beta"));
        assert_eq!(fields, None);

        let mut lines = Some(" \tvalue  # comment\r\nnext\r\n");
        assert_eq!(next_line_strip_comments(&mut lines), Some("value"));
        assert_eq!(next_line_strip_comments(&mut lines), Some("next"));
        assert_eq!(next_line_strip_comments(&mut lines), Some(""));
        assert_eq!(next_line_strip_comments(&mut lines), None);

        let mut words = " \t\"two words\"  tail";
        assert_eq!(next_possibly_quoted_string(&mut words), "two words");
        assert_eq!(words, "tail");
        assert_eq!(next_possibly_quoted_string(&mut words), "tail");
        assert_eq!(words, "");
    }

    #[test]
    fn string_and_path_helpers_match_reference() {
        assert_eq!(to_lower(b'A'), b'a');
        assert_eq!(to_lower(b'Z'), b'z');
        assert_eq!(to_lower(b'!'), b'!');
        let mut owned = String::from("old");
        str_set(&mut owned, "replacement");
        assert_eq!(owned, "replacement");
        assert_eq!(str_fmt(format_args!("{}-{:#x}", "value", 42)), "value-0x2a");
        assert!(string_equals_no_case("MiXeD", "mixed"));
        assert_eq!(string_starts_with_no_case("Zelda3", "zelda"), Some("3"));
        assert_eq!(string_starts_with_no_case("short", "shorter"), None);
        assert_eq!(split_key_value("key \t=  value"), Some(("key", "value")));
        assert_eq!(skip_prefix("prefix-rest", "prefix-"), Some("rest"));
        assert_eq!(
            replace_filename_with_new_path("a/b/file.ini", "new.dat"),
            "a/b/new.dat"
        );
        assert_eq!(
            replace_filename_with_new_path("file.ini", "new.dat"),
            "new.dat"
        );
    }

    #[test]
    fn indexed_memblk_decodes_16_bit_offsets() {
        // Three entries: table offsets 3 and 5, payloads `abc`, `de`, and ``.
        let bytes = [3, 0, 5, 0, b'a', b'b', b'c', b'd', b'e', 2, 0];
        let block = MemBlk::new(&bytes);
        assert_eq!(find_index_in_memblk(block, 0).unwrap().bytes, b"abc");
        assert_eq!(find_index_in_memblk(block, 1).unwrap().bytes, b"de");
        assert_eq!(find_index_in_memblk(block, 2).unwrap().bytes, b"");
        assert!(find_index_in_memblk(block, 3).is_none());
    }

    fn encode_bps_int(mut value: u64, output: &mut Vec<u8>) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                output.push(byte | 0x80);
                return;
            }
            output.push(byte);
            value -= 1;
        }
    }

    fn finish_patch(mut patch: Vec<u8>, source: &[u8], target: &[u8]) -> Vec<u8> {
        patch.extend_from_slice(&crc32(source).to_le_bytes());
        patch.extend_from_slice(&crc32(target).to_le_bytes());
        let patch_crc = crc32(&patch);
        patch.extend_from_slice(&patch_crc.to_le_bytes());
        patch
    }

    #[test]
    fn bps_source_read_and_target_read_match_expected_bytes() {
        let source = b"abc";

        let mut source_read = b"BPS1".to_vec();
        encode_bps_int(3, &mut source_read);
        encode_bps_int(3, &mut source_read);
        encode_bps_int(0, &mut source_read);
        encode_bps_int(8, &mut source_read); // length 3, mode 0
        let source_read = finish_patch(source_read, source, source);
        assert_eq!(apply_bps(source, &source_read).unwrap(), source);

        let mut target_read = b"BPS1".to_vec();
        encode_bps_int(3, &mut target_read);
        encode_bps_int(3, &mut target_read);
        encode_bps_int(0, &mut target_read);
        encode_bps_int(9, &mut target_read); // length 3, mode 1
        target_read.extend_from_slice(b"xyz");
        let target_read = finish_patch(target_read, source, b"xyz");
        assert_eq!(apply_bps(source, &target_read).unwrap(), b"xyz");
    }

    #[test]
    fn crc32_matches_standard_vector_and_bad_patch_is_rejected() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert!(apply_bps(b"abc", b"not a patch").is_none());
    }
}
