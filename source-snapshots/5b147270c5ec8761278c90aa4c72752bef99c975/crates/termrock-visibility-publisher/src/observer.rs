//! In-memory parser for the single-member observer artifact ZIP.

use std::io::{Read, Cursor};

use flate2::read::DeflateDecoder;

const EOCD_SIGNATURE: u32 = 0x0605_4b50;
const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
const DATA_DESCRIPTOR_SIGNATURE: u32 = 0x0807_4b50;
const MAX_ZIP_ENTRIES: usize = 16;
const MAX_ZIP_COMMENT: usize = 65_535;
/// The selected observer producer uploads this exact report path at ZIP root.
/// The current V1 producer lacks observer-run identity fields; the typed
/// ingestion schema requires those V2 additions before it accepts an artifact.
pub const OBSERVER_REPORT_MEMBER: &str = "ci-observer-report.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverError {
    ArchiveTooLarge,
    InvalidZip,
    Zip64Unsupported,
    MultipleMembers,
    UnsafeMemberPath,
    UnsupportedCompression,
    EncryptedMember,
    SymlinkMember,
    MemberTooLarge,
    InvalidDeflate,
    CrcMismatch,
    InvalidJson,
}

impl std::fmt::Display for ObserverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchiveTooLarge => f.write_str("archive exceeded its byte limit"),
            Self::InvalidZip => f.write_str("archive is not a valid bounded ZIP"),
            Self::Zip64Unsupported => f.write_str("ZIP64 archives are unsupported"),
            Self::MultipleMembers => f.write_str("archive must contain exactly one observer member"),
            Self::UnsafeMemberPath => f.write_str("archive member path is not the allowed ci-observer-report.json path"),
            Self::UnsupportedCompression => f.write_str("archive member uses unsupported compression"),
            Self::EncryptedMember => f.write_str("encrypted ZIP members are unsupported"),
            Self::SymlinkMember => f.write_str("symlink and non-regular ZIP members are unsupported"),
            Self::MemberTooLarge => f.write_str("observer JSON exceeded its byte limit"),
            Self::InvalidDeflate => f.write_str("observer ZIP member could not be decompressed safely"),
            Self::CrcMismatch => f.write_str("observer ZIP member CRC did not match"),
            Self::InvalidJson => f.write_str("observer JSON did not match the strict schema"),
        }
    }
}

impl std::error::Error for ObserverError {}

/// Decode only one regular root member named `ci-observer-report.json` into memory.
/// No path from the archive is ever joined to a filesystem path.
pub fn decode_observer_zip(
    archive: &[u8],
    max_archive_bytes: usize,
    max_json_bytes: usize,
) -> Result<Vec<u8>, ObserverError> {
    if archive.len() > max_archive_bytes { return Err(ObserverError::ArchiveTooLarge); }
    let eocd = find_eocd(archive)?;
    let disk = u16_at(archive, eocd + 4)?;
    let central_disk = u16_at(archive, eocd + 6)?;
    let entries_on_disk = u16_at(archive, eocd + 8)?;
    let entries_total = u16_at(archive, eocd + 10)?;
    let central_size = u32_at(archive, eocd + 12)? as usize;
    let central_offset = u32_at(archive, eocd + 16)? as usize;
    let comment_len = u16_at(archive, eocd + 20)? as usize;
    if disk != 0 || central_disk != 0 || entries_on_disk != entries_total {
        return Err(ObserverError::InvalidZip);
    }
    if entries_total == u16::MAX || central_size == u32::MAX as usize || central_offset == u32::MAX as usize {
        return Err(ObserverError::Zip64Unsupported);
    }
    if entries_total == 0 || entries_total as usize > MAX_ZIP_ENTRIES {
        return Err(ObserverError::MultipleMembers);
    }
    if comment_len > MAX_ZIP_COMMENT || eocd + 22 + comment_len != archive.len() {
        return Err(ObserverError::InvalidZip);
    }
    if central_offset.checked_add(central_size) != Some(eocd) {
        return Err(ObserverError::InvalidZip);
    }

    let mut cursor = central_offset;
    let mut members = Vec::with_capacity(entries_total as usize);
    for _ in 0..entries_total {
        if u32_at(archive, cursor)? != CENTRAL_SIGNATURE { return Err(ObserverError::InvalidZip); }
        let made_by = u16_at(archive, cursor + 4)?;
        let flags = u16_at(archive, cursor + 8)?;
        let method = u16_at(archive, cursor + 10)?;
        let crc32 = u32_at(archive, cursor + 16)?;
        let compressed_size = u32_at(archive, cursor + 20)?;
        let uncompressed_size = u32_at(archive, cursor + 24)?;
        let name_len = u16_at(archive, cursor + 28)? as usize;
        let extra_len = u16_at(archive, cursor + 30)? as usize;
        let comment_len = u16_at(archive, cursor + 32)? as usize;
        let disk_start = u16_at(archive, cursor + 34)?;
        let external_attributes = u32_at(archive, cursor + 38)?;
        let local_offset = u32_at(archive, cursor + 42)?;
        if compressed_size == u32::MAX || uncompressed_size == u32::MAX || local_offset == u32::MAX {
            return Err(ObserverError::Zip64Unsupported);
        }
        if flags & 0x0001 != 0 || flags & 0x0040 != 0 { return Err(ObserverError::EncryptedMember); }
        if disk_start != 0 { return Err(ObserverError::InvalidZip); }
        if uncompressed_size as usize > max_json_bytes { return Err(ObserverError::MemberTooLarge); }
        if compressed_size as usize > max_archive_bytes { return Err(ObserverError::ArchiveTooLarge); }
        reject_zip64_extra(archive, cursor + 46 + name_len, extra_len)?;
        let name_start = cursor + 46;
        let name_end = name_start.checked_add(name_len).ok_or(ObserverError::InvalidZip)?;
        let name_bytes = archive.get(name_start..name_end).ok_or(ObserverError::InvalidZip)?;
        if name_bytes.iter().any(|byte| !byte.is_ascii()) { return Err(ObserverError::UnsafeMemberPath); }
        let name = std::str::from_utf8(name_bytes).map_err(|_| ObserverError::UnsafeMemberPath)?;
        if name != OBSERVER_REPORT_MEMBER || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
            return Err(ObserverError::UnsafeMemberPath);
        }
        check_member_type(made_by, external_attributes)?;
        let entry_len = 46usize.checked_add(name_len).and_then(|value| value.checked_add(extra_len))
            .and_then(|value| value.checked_add(comment_len)).ok_or(ObserverError::InvalidZip)?;
        cursor = cursor.checked_add(entry_len).ok_or(ObserverError::InvalidZip)?;
        if cursor > eocd { return Err(ObserverError::InvalidZip); }
        members.push(CentralMember {
            name: name.to_owned(), flags, method, crc32,
            compressed_size: compressed_size as usize,
            uncompressed_size: uncompressed_size as usize,
            local_offset: local_offset as usize,
        });
    }
    if cursor != eocd || members.len() != 1 { return Err(ObserverError::MultipleMembers); }
    let member = members.pop().ok_or(ObserverError::InvalidZip)?;
    extract_member(archive, &member, central_offset, max_json_bytes)
}

#[derive(Debug)]
struct CentralMember {
    name: String,
    flags: u16,
    method: u16,
    crc32: u32,
    compressed_size: usize,
    uncompressed_size: usize,
    local_offset: usize,
}

fn find_eocd(archive: &[u8]) -> Result<usize, ObserverError> {
    if archive.len() < 22 { return Err(ObserverError::InvalidZip); }
    let start = archive.len().saturating_sub(22 + MAX_ZIP_COMMENT);
    for index in (start..=archive.len() - 22).rev() {
        if u32_at(archive, index).ok() == Some(EOCD_SIGNATURE) {
            let comment_len = u16_at(archive, index + 20)? as usize;
            if index + 22 + comment_len == archive.len() { return Ok(index); }
        }
    }
    Err(ObserverError::InvalidZip)
}

fn extract_member(
    archive: &[u8],
    member: &CentralMember,
    central_offset: usize,
    max_json_bytes: usize,
) -> Result<Vec<u8>, ObserverError> {
    let start = member.local_offset;
    if u32_at(archive, start)? != LOCAL_SIGNATURE { return Err(ObserverError::InvalidZip); }
    let flags = u16_at(archive, start + 6)?;
    let method = u16_at(archive, start + 8)?;
    let local_crc = u32_at(archive, start + 14)?;
    let local_compressed = u32_at(archive, start + 18)?;
    let local_uncompressed = u32_at(archive, start + 22)?;
    let name_len = u16_at(archive, start + 26)? as usize;
    let extra_len = u16_at(archive, start + 28)? as usize;
    if flags != member.flags || method != member.method { return Err(ObserverError::InvalidZip); }
    if flags & 0x0001 != 0 || flags & 0x0040 != 0 { return Err(ObserverError::EncryptedMember); }
    reject_zip64_extra(archive, start + 30 + name_len, extra_len)?;
    let name_start = start + 30;
    let name_end = name_start.checked_add(name_len).ok_or(ObserverError::InvalidZip)?;
    let local_name = archive.get(name_start..name_end).ok_or(ObserverError::InvalidZip)?;
    if local_name != member.name.as_bytes() { return Err(ObserverError::InvalidZip); }
    let data_start = name_end.checked_add(extra_len).ok_or(ObserverError::InvalidZip)?;
    let data_end = data_start.checked_add(member.compressed_size).ok_or(ObserverError::InvalidZip)?;
    if data_end > central_offset || member.uncompressed_size > max_json_bytes { return Err(ObserverError::MemberTooLarge); }
    if flags & 0x0008 == 0 {
        if local_crc != member.crc32 || local_compressed as usize != member.compressed_size
            || local_uncompressed as usize != member.uncompressed_size
        {
            return Err(ObserverError::InvalidZip);
        }
    } else {
        validate_data_descriptor(archive, data_end, central_offset, member)?;
    }
    let compressed = archive.get(data_start..data_end).ok_or(ObserverError::InvalidZip)?;
    let output = match member.method {
        0 => {
            if member.compressed_size != member.uncompressed_size { return Err(ObserverError::InvalidZip); }
            compressed.to_vec()
        }
        8 => {
            let decoder = DeflateDecoder::new(Cursor::new(compressed));
            let mut bounded = decoder.take(max_json_bytes as u64 + 1);
            let mut output = Vec::new();
            bounded.read_to_end(&mut output).map_err(|_| ObserverError::InvalidDeflate)?;
            output
        }
        _ => return Err(ObserverError::UnsupportedCompression),
    };
    if output.len() > max_json_bytes || output.len() != member.uncompressed_size {
        return Err(ObserverError::MemberTooLarge);
    }
    if crc32fast::hash(&output) != member.crc32 { return Err(ObserverError::CrcMismatch); }
    Ok(output)
}

fn check_member_type(made_by: u16, external_attributes: u32) -> Result<(), ObserverError> {
    let platform = made_by >> 8;
    if platform == 3 {
        let mode = (external_attributes >> 16) as u16;
        let kind = mode & 0o170000;
        if kind == 0o120000 || (kind != 0 && kind != 0o100000) {
            return Err(ObserverError::SymlinkMember);
        }
    }
    // DOS directory attribute is bit 4; the observer report must be a regular file.
    if external_attributes & 0x10 != 0 { return Err(ObserverError::SymlinkMember); }
    Ok(())
}

fn validate_data_descriptor(
    archive: &[u8],
    start: usize,
    central_offset: usize,
    member: &CentralMember,
) -> Result<(), ObserverError> {
    let mut cursor = start;
    let first = u32_at(archive, cursor)?;
    if first == DATA_DESCRIPTOR_SIGNATURE { cursor += 4; }
    if cursor + 12 > central_offset { return Err(ObserverError::InvalidZip); }
    let crc = u32_at(archive, cursor)?;
    let compressed = u32_at(archive, cursor + 4)?;
    let uncompressed = u32_at(archive, cursor + 8)?;
    if crc != member.crc32 || compressed as usize != member.compressed_size
        || uncompressed as usize != member.uncompressed_size
    {
        return Err(ObserverError::InvalidZip);
    }
    Ok(())
}

fn reject_zip64_extra(archive: &[u8], start: usize, length: usize) -> Result<(), ObserverError> {
    let end = start.checked_add(length).ok_or(ObserverError::InvalidZip)?;
    let extra = archive.get(start..end).ok_or(ObserverError::InvalidZip)?;
    let mut cursor = 0usize;
    while cursor < extra.len() {
        if cursor + 4 > extra.len() { return Err(ObserverError::InvalidZip); }
        let id = u16::from_le_bytes([extra[cursor], extra[cursor + 1]]);
        let size = u16::from_le_bytes([extra[cursor + 2], extra[cursor + 3]]) as usize;
        cursor += 4;
        if id == 0x0001 { return Err(ObserverError::Zip64Unsupported); }
        cursor = cursor.checked_add(size).ok_or(ObserverError::InvalidZip)?;
        if cursor > extra.len() { return Err(ObserverError::InvalidZip); }
    }
    Ok(())
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, ObserverError> {
    let slice = bytes.get(offset..offset + 2).ok_or(ObserverError::InvalidZip)?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, ObserverError> {
    let slice = bytes.get(offset..offset + 4).ok_or(ObserverError::InvalidZip)?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}
