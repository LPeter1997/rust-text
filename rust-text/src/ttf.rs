
// TrueType format interpretation.

use super::parse::*;
use std::collections::{HashMap, HashSet};

/// The magic number that must be in the head table's `magic_number` field.
const HEAD_TABLE_MAGIC: u32 = 0x5F0F3CF5;

// Types defined by Apple, they are just for easier doc-reading.
type Fixed        = i32;
type LongDateTime = i64;
type FWord        = i16;

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
parseable_struct!{OffsetSubtable{
    scaler_type   : u32,
    num_tables    : u16,
    search_range  : u16,
    entry_selector: u16,
    range_shift   : u16,
}}

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
parseable_struct!{TableDirectoryEntry{
    tag     : [u8; 4],
    checksum: u32    ,
    offset  : u32    ,
    length  : u32    ,
}}

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
parseable_struct!{HeadTable{
    version            : Fixed       ,
    font_revision      : Fixed       ,
    checksum_adjustment: u32         ,
    magic_number       : u32         ,
    flags              : u16         ,
    units_per_em       : u16         ,
    created            : LongDateTime,
    modified           : LongDateTime,
    x_min              : FWord       ,
    y_min              : FWord       ,
    x_max              : FWord       ,
    y_max              : FWord       ,
    mac_style          : u16         ,
    lowest_rec_ppem    : u16         ,
    font_direction_hint: i16         ,
    index_to_loc_format: i16         ,
    glyph_data_format  : i16         ,
}}

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
#[repr(C)]
#[derive(Debug, Default, Clone)]
struct NameTable {
    format       : u16            ,
    count        : u16            ,
    string_offset: u16            ,
    name_records : Vec<NameRecord>,
}

impl Parse for NameTable {
    fn parse_be(input: &mut &[u8]) -> ParseResult<Self> {
        let mut bytes = *input;
        let format = Parse::parse_be(&mut bytes)?;
        let count = Parse::parse_be(&mut bytes)?;
        let string_offset = Parse::parse_be(&mut bytes)?;
        let mut name_records: Vec<NameRecord> = Vec::with_capacity(count as usize);
        for _ in 0..count { name_records.push(Parse::parse_be(&mut bytes)?); }
        *input = bytes;
        Ok(Self{
            format,
            count,
            string_offset,
            name_records,
        })
    }
}

// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html
parseable_struct!{NameRecord{
    platform_id         : u16,
    platform_specific_id: u16,
    language_id         : u16,
    name_id             : u16,
    length              : u16,
    offset              : u16,
}}

// TODO: Do we need to store the unused tables?
/// A type that represents a parsed TTF file.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub(crate) struct TtfFile {
    offset: OffsetSubtable,
    head: HeadTable,
    name: NameTable,
    names: HashMap<u16, HashSet<String>>,
}

impl TtfFile {
    /// Parses the bytes into an in-memory TTF structure.
    pub(crate) fn parse(mut input: &[u8]) -> Result<Self, ()> {
        Self::parse_be(&mut input)
    }

    /// Returns the entries with the given NameID from the 'name' table.
    pub(crate) fn name(&self, id: u16) -> Option<&HashSet<String>> {
        self.names.get(&id)
    }
}

impl Parse for TtfFile {
    fn parse_be(input: &mut &[u8]) -> ParseResult<Self> {
        let mut bytes = *input;

        // Initial table
        let offset = OffsetSubtable::parse_be(&mut bytes)?;
        // Collect entries
        let mut entries = HashMap::new();
        for _ in 0..offset.num_tables {
            let e = TableDirectoryEntry::parse_be(&mut bytes).unwrap();
            let tag = format!("{}{}{}{}", e.tag[0] as char, e.tag[1] as char,
                e.tag[2] as char, e.tag[3] as char);
            entries.insert(tag, e);
        }
        // Parse head table
        let head_entry = entries.get("head");
        if head_entry.is_none() {
            return Err(());
        }
        let head_entry = head_entry.unwrap();
        let mut head_bytes = &input[(head_entry.offset as usize)..];
        let head = HeadTable::parse_be(&mut head_bytes)?;
        // Check magic
        if head.magic_number != HEAD_TABLE_MAGIC {
            return Err(());
        }
        // Parse name table
        let name_entry = entries.get("name");
        if name_entry.is_none() {
            return Err(());
        }
        let name_entry = name_entry.unwrap();
        let mut name_bytes = &input[(name_entry.offset as usize)..];
        let orig_name_bytes = name_bytes;
        let name = NameTable::parse_be(&mut name_bytes).unwrap();
        // Collect the names
        let mut names: HashMap<u16, HashSet<String>> = HashMap::new();
        let strings = &orig_name_bytes[(name.string_offset as usize)..];
        for e in &name.name_records {
            let offs = e.offset as usize;
            let len = e.length as usize;
            // Byte sequence for the string
            let data = &strings[offs..(offs + len)];
            let text = if e.platform_id == 1 {
                    // ASCII
                    String::from_utf8_lossy(data).into_owned()
                }
                else {
                    // UTF16
                    let text16: Vec<_> = data
                        .chunks_exact(2)
                        .into_iter()
                        .map(|a| u16::from_be_bytes([a[0], a[1]]))
                        .collect();
                    String::from_utf16_lossy(&text16)
                };
            // Add it to the names
            if !names.contains_key(&e.name_id) {
                names.insert(e.name_id, HashSet::new());
            }
            let ns = names.get_mut(&e.name_id).unwrap();
            ns.insert(text);
        }

        *input = bytes;

        Ok(Self{
            offset,
            head,
            name,
            names,
        })
    }
}
