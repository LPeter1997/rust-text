
// TrueType format interpretation.

use super::parse::*;
use std::collections::HashMap;

const HEAD_TABLE_MAGIC: u32 = 0x5F0F3CF5;

type Fixed        = i32;
type LongDateTime = i64;
type FWord        = i16;

parseable_struct!{OffsetSubtable{
    scaler_type   : u32,
    num_tables    : u16,
    search_range  : u16,
    entry_selector: u16,
    range_shift   : u16,
}}

parseable_struct!{TableDirectoryEntry{
    tag     : [u8; 4],
    checksum: u32    ,
    offset  : u32    ,
    length  : u32    ,
}}

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
    _padding           : i16         ,
}}

fn long_align(origin: &[u8], input: &mut &[u8]) -> Result<(), ()> {
    let actual = *input;
    let distance = actual.as_ptr() as usize - origin.as_ptr() as usize;
    // TODO: Apple doc says long-align. Is that 4 bytes?
    const LONG_ALIGNMENT: usize = 4;
    let mut required_step = LONG_ALIGNMENT - distance % LONG_ALIGNMENT;
    if required_step == LONG_ALIGNMENT {
        required_step = 0;
    }
    if actual.len() < required_step {
        return Err(());
    }
    *input = actual;
    Ok(())
}

pub fn parse_ttf(input: &[u8]) {
    let mut bytes = input;

    // Initial table
    let offset = OffsetSubtable::parse_be(&mut bytes).unwrap();

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
        return;
    }
    let head_entry = head_entry.unwrap();
    let mut head_bytes = &input[(head_entry.offset as usize)..];
    long_align(input, &mut head_bytes).unwrap();
    let head = HeadTable::parse_be(&mut head_bytes).unwrap();
    println!("{:?}", head);
}
