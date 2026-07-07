use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

/// Parse resources.arsc to extract string pool and package info
pub struct ArscParser;

#[derive(Debug, Clone, Default)]
pub struct ArscData {
    pub string_pool: Vec<String>,
    pub packages: Vec<PackageData>,
}

#[derive(Debug, Clone, Default)]
pub struct PackageData {
    pub package_id: u32,
    pub package_name: String,
    pub type_strings: Vec<String>,
    pub key_strings: Vec<String>,
}

impl ArscParser {
    pub fn parse(data: &[u8]) -> Result<ArscData, String> {
        let mut cursor = Cursor::new(data);
        let mut result = ArscData::default();

        // Read RES_TABLE_TYPE
        let table_type = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        if table_type != 0x0002 {
            return Err("Not a valid resources.arsc".to_string());
        }
        let _header_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        let _chunk_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let package_count = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        let mut global_strings: Vec<String> = Vec::new();

        for _ in 0..package_count {
            let pos = cursor.position() as usize;
            if pos + 8 > data.len() {
                break;
            }

            let chunk_type = cursor.read_u16::<LittleEndian>().unwrap_or(0);
            let _header_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
            let chunk_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);

            let chunk_end = pos + chunk_size as usize;

            if chunk_type == 0x0001 {
                // String pool
                global_strings = parse_string_pool_raw(&data[pos..chunk_end])?;
            } else if chunk_type == 0x0200 {
                // Package
                let pkg = parse_package(&data[pos..chunk_end])?;
                result.packages.push(pkg);
            }

            cursor.set_position(chunk_end as u64);
        }

        result.string_pool = global_strings;
        Ok(result)
    }
}

fn parse_string_pool_raw(data: &[u8]) -> Result<Vec<String>, String> {
    let mut cursor = Cursor::new(data);
    let _chunk_type = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _header_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _chunk_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);

    let string_count = cursor.read_u32::<LittleEndian>().unwrap_or(0) as usize;
    let _style_count = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let flags = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let strings_offset = cursor.read_u32::<LittleEndian>().unwrap_or(0) as usize;
    let _styles_offset = cursor.read_u32::<LittleEndian>().unwrap_or(0);

    let is_utf8 = (flags & (1 << 8)) != 0;

    let mut offsets = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        offsets.push(cursor.read_u32::<LittleEndian>().unwrap_or(0) as usize);
    }

    let strings_start = strings_offset;
    let mut strings = Vec::with_capacity(string_count);

    for i in 0..string_count {
        let offset = strings_start + offsets.get(i).unwrap_or(&0);
        if offset >= data.len() {
            strings.push(String::new());
            continue;
        }
        let s = if is_utf8 {
            read_utf8(data, offset)
        } else {
            read_utf16(data, offset)
        };
        strings.push(s);
    }

    Ok(strings)
}

fn read_utf8(data: &[u8], offset: usize) -> String {
    if offset >= data.len() {
        return String::new();
    }
    let len_byte = data[offset];
    let mut pos = offset + 1;
    let len = if len_byte & 0x80 != 0 {
        if pos >= data.len() {
            return String::new();
        }
        let len_byte2 = data[pos];
        pos += 1;
        ((len_byte as usize & 0x7F) << 8) | len_byte2 as usize
    } else {
        len_byte as usize
    };

    // Skip encoded length byte(s)
    if pos >= data.len() {
        return String::new();
    }
    let _enc_len = data[pos];
    pos += 1;
    if _enc_len & 0x80 != 0 {
        pos += 1;
    }

    if pos + len > data.len() {
        return String::new();
    }
    String::from_utf8_lossy(&data[pos..pos + len]).to_string()
}

fn read_utf16(data: &[u8], offset: usize) -> String {
    if offset + 2 > data.len() {
        return String::new();
    }
    let len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
    let pos = offset + 2;
    let byte_len = len * 2;
    if pos + byte_len > data.len() {
        return String::new();
    }
    let u16_data: Vec<u16> = data[pos..pos + byte_len]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&u16_data)
}

fn parse_package(data: &[u8]) -> Result<PackageData, String> {
    let mut cursor = Cursor::new(data);
    let _chunk_type = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _header_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _chunk_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let package_id = cursor.read_u32::<LittleEndian>().unwrap_or(0);

    // Package name: 128 UTF-16 chars (256 bytes)
    let mut name_bytes = [0u8; 256];
    cursor.read_exact(&mut name_bytes).unwrap_or(());
    let name_u16: Vec<u16> = name_bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&c| c != 0)
        .collect();
    let package_name = String::from_utf16_lossy(&name_u16);

    let _type_strings_offset = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let _last_public_type = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let _key_strings_offset = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let _last_public_key = cursor.read_u32::<LittleEndian>().unwrap_or(0);

    Ok(PackageData {
        package_id,
        package_name,
        type_strings: Vec::new(),
        key_strings: Vec::new(),
    })
}
