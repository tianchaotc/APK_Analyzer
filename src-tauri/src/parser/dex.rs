use std::collections::HashMap;
use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};

/// DEX file header
const DEX_MAGIC: &[u8] = b"dex\n";

/// Parse a DEX file to extract class, method, field statistics
pub struct DexParser;

impl DexParser {
    pub fn parse(data: &[u8]) -> Result<DexStats, String> {
        if data.len() < 112 {
            return Err("DEX file too small".to_string());
        }

        if &data[0..4] != DEX_MAGIC {
            return Err("Invalid DEX magic".to_string());
        }

        let mut cursor = Cursor::new(data);

        // Skip magic (8 bytes: "dex\n" + version 3 bytes + "\n")
        cursor.set_position(8);

        // Checksum (4) + Signature (20)
        cursor.set_position(32);

        // File size
        let file_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Header size
        let _header_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Endian tag
        let _endian_tag = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Link size, link offset
        cursor.set_position(48);

        // Map offset
        cursor.set_position(52);
        let _map_offset = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // String IDs
        let string_ids_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let _string_ids_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Type IDs
        let type_ids_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let _type_ids_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Proto IDs
        let proto_ids_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        cursor.set_position(76);

        // Field IDs
        let field_ids_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let _field_ids_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Method IDs
        let method_ids_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let _method_ids_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Class defs
        let class_defs_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let class_defs_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Data size, data_off
        let _data_size = cursor.read_u32::<LittleEndian>().unwrap_or(0);
        let _data_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        // Parse string table
        let strings = if string_ids_size > 0 {
            cursor.set_position(56);
            let string_ids_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);
            parse_string_table(data, string_ids_off, string_ids_size)
        } else {
            Vec::new()
        };

        // Parse type table
        let types = if type_ids_size > 0 {
            cursor.set_position(64);
            let type_ids_off = cursor.read_u32::<LittleEndian>().unwrap_or(0);
            parse_type_table(data, type_ids_off, type_ids_size, &strings)
        } else {
            Vec::new()
        };

        // Parse class defs
        let (class_names, package_info) = if class_defs_size > 0 {
            parse_class_defs(data, class_defs_off, class_defs_size, &types, &strings)
        } else {
            (Vec::new(), HashMap::new())
        };

        Ok(DexStats {
            file_size: file_size as u64,
            class_count: class_defs_size as usize,
            method_count: method_ids_size as usize,
            field_count: field_ids_size as usize,
            string_count: string_ids_size as usize,
            type_count: type_ids_size as usize,
            proto_count: proto_ids_size as usize,
            class_names,
            packages: package_info,
        })
    }
}

fn parse_string_table(data: &[u8], offset: u32, count: u32) -> Vec<String> {
    let mut strings = Vec::with_capacity(count as usize);
    let mut pos = offset as usize;

    for _ in 0..count {
        if pos + 4 > data.len() {
            break;
        }
        let str_data_off = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
        ]) as usize;
        pos += 4;

        let s = read_uleb128_string(data, str_data_off);
        strings.push(s);
    }

    strings
}

fn read_uleb128_string(data: &[u8], offset: usize) -> String {
    if offset >= data.len() {
        return String::new();
    }

    // Read ULEB128 length
    let (len, bytes_read) = read_uleb128(data, offset);
    let str_start = offset + bytes_read;
    let str_len = len as usize;

    if str_start + str_len > data.len() {
        return String::new();
    }

    // MUTF-8 encoded string
    String::from_utf8_lossy(&data[str_start..str_start + str_len]).to_string()
}

fn read_uleb128(data: &[u8], offset: usize) -> (u64, usize) {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut pos = offset;

    loop {
        if pos >= data.len() {
            return (result, pos - offset);
        }
        let byte = data[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    (result, pos - offset)
}

fn parse_type_table(data: &[u8], offset: u32, count: u32, strings: &[String]) -> Vec<String> {
    let mut types = Vec::with_capacity(count as usize);
    let mut pos = offset as usize;

    for _ in 0..count {
        if pos + 4 > data.len() {
            break;
        }
        let descriptor_idx = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
        ]) as usize;
        pos += 4;

        let type_str = strings.get(descriptor_idx).cloned().unwrap_or_default();
        types.push(type_str);
    }

    types
}

fn parse_class_defs(
    data: &[u8],
    offset: u32,
    count: u32,
    types: &[String],
    strings: &[String],
) -> (Vec<String>, HashMap<String, crate::models::dex::PackageInfo>) {
    let mut class_names = Vec::with_capacity(count as usize);
    let mut packages: HashMap<String, crate::models::dex::PackageInfo> = HashMap::new();

    let mut pos = offset as usize;
    let class_def_size = 32; // Each class_def_item is 32 bytes

    for _ in 0..count {
        if pos + class_def_size > data.len() {
            break;
        }

        let class_idx = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
        ]) as usize;
        pos += class_def_size;

        let class_descriptor = types.get(class_idx).cloned().unwrap_or_default();

        // Convert type descriptor (Lcom/example/Foo;) to class name
        let class_name = descriptor_to_classname(&class_descriptor);
        class_names.push(class_name.clone());

        // Extract package
        if let Some(pkg) = extract_package(&class_descriptor) {
            let entry = packages.entry(pkg.clone()).or_insert_with(|| {
                crate::models::dex::PackageInfo {
                    name: pkg,
                    class_count: 0,
                    method_count: 0,
                    field_count: 0,
                }
            });
            entry.class_count += 1;
        }
    }

    (class_names, packages)
}

fn descriptor_to_classname(descriptor: &str) -> String {
    if descriptor.starts_with('L') && descriptor.ends_with(';') {
        let inner = &descriptor[1..descriptor.len() - 1];
        inner.replace('/', ".")
    } else {
        descriptor.to_string()
    }
}

fn extract_package(descriptor: &str) -> Option<String> {
    if !descriptor.starts_with('L') || !descriptor.ends_with(';') {
        return None;
    }
    let inner = &descriptor[1..descriptor.len() - 1];
    let path = inner.replace('/', ".");
    let parts: Vec<&str> = path.rsplitn(2, '.').collect();
    if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        Some("(default)".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct DexStats {
    pub file_size: u64,
    pub class_count: usize,
    pub method_count: usize,
    pub field_count: usize,
    pub string_count: usize,
    pub type_count: usize,
    pub proto_count: usize,
    pub class_names: Vec<String>,
    pub packages: HashMap<String, crate::models::dex::PackageInfo>,
}
