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
    let chunk_type = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| format!("Failed to read chunk type: {}", e))?;
    if chunk_type != CHUNK_AXML_FILE {
        return Err(format!("Invalid AXML magic: 0x{:04X}", chunk_type));
    }
    let _header_size = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| format!("Failed to read header size: {}", e))?;
    let _file_size = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| format!("Failed to read file size: {}", e))?;

    // Parse chunks
    let mut strings: Vec<String> = Vec::new();
    let mut resource_map: Vec<u32> = Vec::new();
    // Stack-based element parsing
    let mut root: Option<AxmlElement> = None;
    let mut stack: Vec<AxmlElement> = Vec::new();

    while (cursor.position() as usize) < data.len() {
        let pos = cursor.position() as usize;
        if pos + 8 > data.len() {
            break;
        }

        let chunk_type = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| format!("Failed to read chunk type at {}: {}", pos, e))?;
        let _header_size = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| format!("Failed to read header size: {}", e))?;
        let chunk_size = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| format!("Failed to read chunk size: {}", e))?;

        if chunk_size < 8 {
            return Err(format!(
                "AXML: invalid chunk size {} at {}",
                chunk_size, pos
            ));
        }
        let chunk_end = pos
            .checked_add(chunk_size as usize)
            .ok_or_else(|| format!("AXML: chunk size overflow at {}", pos))?;
        if chunk_end > data.len() {
            return Err(format!("AXML: chunk at {} extends past input", pos));
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
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_END_NAMESPACE => {
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_START_TAG => {
                let element = parse_start_tag(&mut cursor, &strings, &resource_map, chunk_end)?;
                stack.push(element);
                cursor.set_position(chunk_end as u64);
            }
            CHUNK_XML_END_TAG => {
                cursor.set_position(chunk_end as u64);
                if let Some(elem) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(elem);
                    } else {
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

    if data.len() < 28 {
        return Err("AXML: truncated string pool header".to_string());
    }

    let _chunk_type = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let _header_size = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let _chunk_size = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;

    let string_count = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let _style_count = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let flags = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let strings_offset = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let _styles_offset = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;

    let offsets_end = 28usize
        .checked_add(string_count.saturating_mul(4))
        .ok_or_else(|| "AXML: string offset table overflow".to_string())?;
    if offsets_end > data.len() || strings_offset > data.len() {
        return Err("AXML: invalid string pool bounds".to_string());
    }

    let is_utf8 = (flags & (1 << 8)) != 0;

    // Read string offsets
    let mut offsets = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        offsets.push(
            cursor
                .read_u32::<LittleEndian>()
                .map_err(|e| e.to_string())? as usize,
        );
    }

    let strings_start = strings_offset;
    let mut strings = Vec::with_capacity(string_count);

    for i in 0..string_count {
        let offset = match strings_start.checked_add(offsets[i]) {
            Some(offset) => offset,
            None => {
                strings.push(String::new());
                continue;
            }
        };
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
    if data.len() < 2 {
        return String::new();
    }

    let mut pos = 0;
    // Read length (1 or 2 bytes)
    let len_byte = data[pos];
    pos += 1;
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

    // Skip the second length field (same encoding, for UTF-8 encoded bytes count)
    if pos >= data.len() {
        return String::new();
    }
    let _encoded_len_byte = data[pos];
    pos += 1;
    if _encoded_len_byte & 0x80 != 0 {
        if pos >= data.len() {
            return String::new();
        }
        pos += 1;
    }

    if pos.checked_add(len).map_or(true, |end| end > data.len()) {
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

    let byte_len = match len.checked_mul(2) {
        Some(byte_len) => byte_len,
        None => return String::new(),
    };
    if pos
        .checked_add(byte_len)
        .map_or(true, |end| end > data.len())
    {
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
    let start = cursor.position() as usize;
    if start.checked_add(28).map_or(true, |end| end > _chunk_end) {
        return Err("AXML: truncated start tag".to_string());
    }

    let _line = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let _comment = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let ns_idx = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let name_idx = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;

    let _attr_start = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let attr_size = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let attr_count = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())? as usize;
    let _id_idx = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let _class_idx = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    let _style_idx = cursor
        .read_u16::<LittleEndian>()
        .map_err(|e| e.to_string())?;

    let attr_size = attr_size.max(20);
    let attrs_end = cursor
        .position()
        .checked_add((attr_count.saturating_mul(attr_size)) as u64)
        .and_then(|pos| usize::try_from(pos).ok())
        .ok_or_else(|| "AXML: attribute table overflow".to_string())?;
    if attrs_end > _chunk_end {
        return Err("AXML: attributes extend past start tag".to_string());
    }

    let name = strings.get(name_idx).cloned().unwrap_or_default();
    let namespace = if ns_idx == 0xFFFFFFFF {
        None
    } else {
        strings.get(ns_idx).cloned()
    };

    let mut attributes = Vec::with_capacity(attr_count);

    for _ in 0..attr_count {
        let attr_ns_idx = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let attr_name_idx = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let attr_raw_value_idx = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;

        let _typed_size = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| e.to_string())?;
        let _typed_res0 = cursor.read_u8().map_err(|e| e.to_string())?;
        let typed_type = cursor.read_u8().map_err(|e| e.to_string())?;
        let typed_data = cursor
            .read_u32::<LittleEndian>()
            .map_err(|e| e.to_string())?;

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
            if typed_data != 0 {
                "true".to_string()
            } else {
                "false".to_string()
            }
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
        let escaped = attr
            .value
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

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u16(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn string_pool(strings: &[&str]) -> Vec<u8> {
        let header_size = 28usize;
        let offsets_size = strings.len() * 4;
        let strings_offset = header_size + offsets_size;
        let mut string_data = Vec::new();
        let mut offsets = Vec::new();

        for value in strings {
            offsets.push(string_data.len() as u32);
            string_data.push(value.len() as u8);
            string_data.push(value.len() as u8);
            string_data.extend_from_slice(value.as_bytes());
            string_data.push(0);
        }

        let chunk_size = strings_offset + string_data.len();
        let mut out = Vec::new();
        push_u16(&mut out, CHUNK_STRING_POOL);
        push_u16(&mut out, header_size as u16);
        push_u32(&mut out, chunk_size as u32);
        push_u32(&mut out, strings.len() as u32);
        push_u32(&mut out, 0);
        push_u32(&mut out, 1 << 8);
        push_u32(&mut out, strings_offset as u32);
        push_u32(&mut out, 0);
        for offset in offsets {
            push_u32(&mut out, offset);
        }
        out.extend_from_slice(&string_data);
        out
    }

    fn start_tag(name_idx: u32) -> Vec<u8> {
        let mut out = Vec::new();
        push_u16(&mut out, CHUNK_XML_START_TAG);
        push_u16(&mut out, 16);
        push_u32(&mut out, 36);
        push_u32(&mut out, 0);
        push_u32(&mut out, 0xFFFF_FFFF);
        push_u32(&mut out, 0xFFFF_FFFF);
        push_u32(&mut out, name_idx);
        push_u16(&mut out, 20);
        push_u16(&mut out, 20);
        push_u16(&mut out, 0);
        push_u16(&mut out, 0);
        push_u16(&mut out, 0);
        push_u16(&mut out, 0);
        out
    }

    fn end_tag() -> Vec<u8> {
        let mut out = Vec::new();
        push_u16(&mut out, CHUNK_XML_END_TAG);
        push_u16(&mut out, 16);
        push_u32(&mut out, 24);
        out.resize(24, 0);
        out
    }

    #[test]
    fn decode_preserves_nested_children_when_end_tags_close() {
        let mut data = Vec::new();
        push_u16(&mut data, CHUNK_AXML_FILE);
        push_u16(&mut data, 8);
        push_u32(&mut data, 0);
        data.extend_from_slice(&string_pool(&["manifest", "application", "activity"]));
        data.extend_from_slice(&start_tag(0));
        data.extend_from_slice(&start_tag(1));
        data.extend_from_slice(&start_tag(2));
        data.extend_from_slice(&end_tag());
        data.extend_from_slice(&end_tag());
        data.extend_from_slice(&end_tag());

        let root = decode(&data).expect("valid synthetic AXML should decode");

        assert_eq!(root.name, "manifest");
        assert_eq!(root.children.len(), 1);
        assert_eq!(root.children[0].name, "application");
        assert_eq!(root.children[0].children.len(), 1);
        assert_eq!(root.children[0].children[0].name, "activity");
    }

    #[test]
    fn read_utf8_string_returns_empty_when_length_field_is_truncated() {
        let result = std::panic::catch_unwind(|| read_utf8_string(&[0x81]));

        assert!(result.is_ok());
        assert_eq!(result.unwrap_or_default(), "");
    }
}
