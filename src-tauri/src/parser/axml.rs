use std::collections::HashMap;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

/// AXML chunk types
const CHUNK_AXML_FILE: u16 = 0x0003;
const CHUNK_STRING_POOL: u16 = 0x0001;
const CHUNK_RESOURCE_MAP: u16 = 0x0180;
const CHUNK_XML_START_NAMESPACE: u16 = 0x0100;
const CHUNK_XML_END_NAMESPACE: u16 = 0x0101;
const CHUNK_XML_START_TAG: u16 = 0x0102;
const CHUNK_XML_END_TAG: u16 = 0x0103;
const CHUNK_XML_CDATA: u16 = 0x0104;

const ATTR_TYPE_NULL: u8 = 0x00;
const ATTR_TYPE_REFERENCE: u8 = 0x01;
const ATTR_TYPE_ATTRIBUTE: u8 = 0x02;
const ATTR_TYPE_STRING: u8 = 0x03;
const ATTR_TYPE_FLOAT: u8 = 0x04;
const ATTR_TYPE_DIMENSION: u8 = 0x05;
const ATTR_TYPE_FRACTION: u8 = 0x06;
const ATTR_TYPE_INT_DEC: u8 = 0x10;
const ATTR_TYPE_INT_HEX: u8 = 0x11;
const ATTR_TYPE_INT_BOOL: u8 = 0x12;

/// Parsed AXML element
#[derive(Debug, Clone)]
pub struct AxmlElement {
    pub name: String,
    pub namespace: Option<String>,
    pub attributes: Vec<AxmlAttribute>,
    pub children: Vec<AxmlElement>,
    pub text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AxmlAttribute {
    pub name: String,
    pub namespace: Option<String>,
    pub value: String,
    pub raw_value: Option<String>,
    pub typed_type: u8,
}

/// Decode binary Android XML to a structured element tree
pub fn decode(data: &[u8]) -> Result<AxmlElement, String> {
    let mut cursor = Cursor::new(data);

    // Read file header
    let chunk_type = cursor.read_u16::<LittleEndian>()
        .map_err(|e| format!("Failed to read chunk type: {}", e))?;
    if chunk_type != CHUNK_AXML_FILE {
        return Err(format!("Invalid AXML magic: 0x{:04X}", chunk_type));
    }
    let _header_size = cursor.read_u16::<LittleEndian>()
        .map_err(|e| format!("Failed to read header size: {}", e))?;
    let _file_size = cursor.read_u32::<LittleEndian>()
        .map_err(|e| format!("Failed to read file size: {}", e))?;

    // Parse chunks
    let mut strings: Vec<String> = Vec::new();
    let mut resource_map: Vec<u32> = Vec::new();
    let mut namespaces: HashMap<String, String> = HashMap::new();

    // Stack-based element parsing
    let mut root: Option<AxmlElement> = None;
    let mut stack: Vec<AxmlElement> = Vec::new();

    while (cursor.position() as usize) < data.len() {
        let pos = cursor.position() as usize;
        if pos + 8 > data.len() {
            break;
        }

        let chunk_type = cursor.read_u16::<LittleEndian>()
            .map_err(|e| format!("Failed to read chunk type at {}: {}", pos, e))?;
        let _header_size = cursor.read_u16::<LittleEndian>()
            .map_err(|e| format!("Failed to read header size: {}", e))?;
        let chunk_size = cursor.read_u32::<LittleEndian>()
            .map_err(|e| format!("Failed to read chunk size: {}", e))?;

        let chunk_end = pos + chunk_size as usize;
        if chunk_end > data.len() {
            break;
        }

        match chunk_type {
            CHUNK_STRING_POOL => {
                strings = parse_string_pool(&data[pos..chunk_end])?;
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_RESOURCE_MAP => {
                resource_map = parse_resource_map(&data[pos + 8..chunk_end]);
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_START_NAMESPACE => {
                let _line = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                let _comment = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                let prefix_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;
                let uri_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;
                let prefix = strings.get(prefix_idx).cloned().unwrap_or_default();
                let uri = strings.get(uri_idx).cloned().unwrap_or_default();
                namespaces.insert(uri, prefix);
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_END_NAMESPACE => {
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_START_TAG => {
                let element = parse_start_tag(&mut cursor, &strings, &resource_map, chunk_end)?;
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(element.clone());
                    stack.push(element);
                } else {
                    stack.push(element);
                }
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_END_TAG => {
                cursor.set_position(chunk_end as u64);
                if let Some(elem) = stack.pop() {
                    if stack.is_empty() {
                        root = Some(elem);
                    }
                }
            }
            CHUNK_XML_CDATA => {
                let _line = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                let _comment = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                let data_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;
                let _typed_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
                let _typed_res0 = cursor.read_u8().unwrap_or(0);
                let _typed_type = cursor.read_u8().unwrap_or(0);
                let _typed_data = cursor.read_u32::<LittleEndian>().unwrap_or(0);

                let text = strings.get(data_idx).cloned();
                if let Some(parent) = stack.last_mut() {
                    parent.text = text;
                }
                cursor.set_position(chunk_end as u64);
            }
            _ => {
                cursor.set_position(chunk_end as u64);
            }
        }
    }

    // If root is not set but stack has one element (the root wasn't closed with end tag properly)
    if root.is_none() && stack.len() == 1 {
        root = stack.pop();
    }

    root.ok_or_else(|| "AXML: No root element found".to_string())
}

fn parse_string_pool(data: &[u8]) -> Result<Vec<String>, String> {
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

    // Read string offsets
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
            read_utf8_string(&data[offset..])
        } else {
            read_utf16_string(&data[offset..])
        };
        strings.push(s);
    }

    Ok(strings)
}

fn read_utf8_string(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut pos = 0;
    // Read length (1 or 2 bytes)
    let len_byte = data[pos];
    pos += 1;
    let len = if len_byte & 0x80 != 0 {
        let len_byte2 = data[pos];
        pos += 1;
        ((len_byte as usize & 0x7F) << 8) | len_byte2 as usize
    } else {
        len_byte as usize
    };

    // Skip the second length field (same encoding, for UTF-8 encoded bytes count)
    let _encoded_len_byte = data[pos];
    pos += 1;
    if _encoded_len_byte & 0x80 != 0 {
        pos += 1;
    }

    if pos + len > data.len() {
        return String::new();
    }

    String::from_utf8_lossy(&data[pos..pos + len]).to_string()
}

fn read_utf16_string(data: &[u8]) -> String {
    if data.len() < 2 {
        return String::new();
    }

    let mut pos = 0;
    let len_word = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2;
    let len = if len_word & 0x8000 != 0 {
        if data.len() < pos + 2 {
            return String::new();
        }
        let len_word2 = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
        pos += 2;
        ((len_word & 0x7FFF) << 16) | len_word2
    } else {
        len_word
    };

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

fn parse_resource_map(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

fn parse_start_tag(
    cursor: &mut Cursor<&[u8]>,
    strings: &[String],
    _resource_map: &[u32],
    _chunk_end: usize,
) -> Result<AxmlElement, String> {
    let _line = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let _comment = cursor.read_u32::<LittleEndian>().unwrap_or(0);
    let ns_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;
    let name_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;

    let _attr_start = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _attr_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let attr_count = cursor.read_u16::<LittleEndian>().unwrap_or(0) as usize;
    let _id_idx = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _class_idx = cursor.read_u16::<LittleEndian>().unwrap_or(0);
    let _style_idx = cursor.read_u16::<LittleEndian>().unwrap_or(0);

    let name = strings.get(name_idx).cloned().unwrap_or_default();
    let namespace = if ns_idx == 0xFFFFFFFF {
        None
    } else {
        strings.get(ns_idx).cloned()
    };

    let mut attributes = Vec::with_capacity(attr_count);

    for _ in 0..attr_count {
        let attr_ns_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;
        let attr_name_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;
        let attr_raw_value_idx = cursor.read_u32::<LittleEndian>().unwrap_or(0xFFFFFFFF) as usize;

        let _typed_size = cursor.read_u16::<LittleEndian>().unwrap_or(0);
        let _typed_res0 = cursor.read_u8().unwrap_or(0);
        let typed_type = cursor.read_u8().unwrap_or(0);
        let typed_data = cursor.read_u32::<LittleEndian>().unwrap_or(0);

        let attr_name = strings.get(attr_name_idx).cloned().unwrap_or_default();
        let attr_ns = if attr_ns_idx == 0xFFFFFFFF {
            None
        } else {
            strings.get(attr_ns_idx).cloned()
        };

        let raw_value = if attr_raw_value_idx == 0xFFFFFFFF {
            None
        } else {
            strings.get(attr_raw_value_idx).cloned()
        };

        let value = if let Some(ref raw) = raw_value {
            raw.clone()
        } else {
            type_to_string(typed_type, typed_data)
        };

        attributes.push(AxmlAttribute {
            name: attr_name,
            namespace: attr_ns,
            value,
            raw_value,
            typed_type,
        });
    }

    Ok(AxmlElement {
        name,
        namespace,
        attributes,
        children: Vec::new(),
        text: None,
    })
}

fn type_to_string(typed_type: u8, typed_data: u32) -> String {
    match typed_type {
        ATTR_TYPE_NULL => String::new(),
        ATTR_TYPE_REFERENCE => format!("@{}", typed_data),
        ATTR_TYPE_ATTRIBUTE => format!("?{}", typed_data),
        ATTR_TYPE_STRING => String::new(),
        ATTR_TYPE_FLOAT => {
            let bits = typed_data;
            format!("{}", f32::from_bits(bits))
        }
        ATTR_TYPE_INT_DEC => format!("{}", typed_data as i32),
        ATTR_TYPE_INT_HEX => format!("0x{:08X}", typed_data),
        ATTR_TYPE_INT_BOOL => {
            if typed_data != 0 { "true".to_string() } else { "false".to_string() }
        }
        ATTR_TYPE_DIMENSION => format!("dimension({})", typed_data),
        ATTR_TYPE_FRACTION => format!("fraction({})", typed_data),
        _ => format!("{}", typed_data),
    }
}

/// Convert an AXML element tree to pretty-printed XML text
pub fn to_xml(element: &AxmlElement) -> String {
    let mut output = String::new();
    output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    element_to_xml(element, &mut output, 0);
    output
}

fn element_to_xml(element: &AxmlElement, output: &mut String, indent: usize) {
    let pad = "  ".repeat(indent);
    output.push_str(&pad);
    output.push('<');
    output.push_str(&element.name);

    for attr in &element.attributes {
        output.push(' ');
        output.push_str(&attr.name);
        output.push_str("=\"");
        // Escape XML special chars
        let escaped = attr.value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;");
        output.push_str(&escaped);
        output.push('"');
    }

    if element.children.is_empty() && element.text.is_none() {
        output.push_str(" />\n");
    } else {
        output.push('>');
        output.push('\n');

        if let Some(ref text) = element.text {
            if !text.is_empty() {
                let inner_pad = "  ".repeat(indent + 1);
                output.push_str(&inner_pad);
                output.push_str(text);
                output.push('\n');
            }
        }

        for child in &element.children {
            element_to_xml(child, output, indent + 1);
        }

        output.push_str(&pad);
        output.push_str("</");
        output.push_str(&element.name);
        output.push_str(">\n");
    }
}
