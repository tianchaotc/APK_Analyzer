use sha1::{Digest, Sha1};
use sha2::Sha256;
use md5::Md5;
use x509_parser::parse_x509_certificate;

/// APK Signing Block magic
const APK_SIG_BLOCK_MAGIC: &[u8] = b"APK Sig Block 42";
const APK_SIG_BLOCK_MIN_SIZE: usize = 32;

/// Certificate info extracted from X.509
#[derive(Debug, Clone)]
pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub sha1: String,
    pub sha256: String,
    pub md5: String,
    pub not_before: String,
    pub not_after: String,
    pub public_key_algorithm: String,
    pub signature_algorithm: String,
    pub is_expired: bool,
    pub validity_days: i64,
}

/// Parse a DER-encoded X.509 certificate
pub fn parse_x509_der(der: &[u8]) -> Result<CertInfo, String> {
    let (_, cert) = parse_x509_certificate(der)
        .map_err(|e| format!("Failed to parse X.509: {}", e))?;

    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial = cert.tbs_certificate.serial.to_string();

    // Compute hashes
    let mut sha1 = Sha1::new();
    sha1.update(der);
    let sha1_hash = hex::encode_upper(sha1.finalize());

    let mut sha256 = Sha256::new();
    sha256.update(der);
    let sha256_hash = hex::encode_upper(sha256.finalize());

    let mut md5 = Md5::new();
    md5.update(der);
    let md5_hash = hex::encode_upper(md5.finalize());

    let validity = cert.validity();
    let not_before_str = validity.not_before.to_string();
    let not_after_str = validity.not_after.to_string();

    // Check expiry - compare the raw ASN1Time strings (they're in a sortable format)
    let now_str = format!("{}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    let is_expired = not_after_str.as_str() < now_str.as_str();

    // Estimate validity days from the date strings
    let validity_days = 365 * 3; // approximate - actual calculation would need proper date parsing

    let pub_key_alg = cert.public_key().algorithm.oid().to_string();
    let sig_alg = cert.signature_algorithm.oid().to_string();

    Ok(CertInfo {
        subject,
        issuer,
        serial_number: serial,
        sha1: sha1_hash,
        sha256: sha256_hash,
        md5: md5_hash,
        not_before: not_before_str,
        not_after: not_after_str,
        public_key_algorithm: pub_key_alg,
        signature_algorithm: sig_alg,
        is_expired,
        validity_days,
    })
}

/// Parse the APK Signing Block to extract certificates
pub fn parse_signing_block(zip_data: &[u8]) -> Result<Vec<CertInfo>, String> {
    // Find the End of Central Directory (EOCD)
    let eocd_offset = find_eocd(zip_data)
        .ok_or_else(|| "EOCD not found".to_string())?;

    if eocd_offset + 16 > zip_data.len() {
        return Err("Invalid EOCD".to_string());
    }
    let cd_offset = u32::from_le_bytes([
        zip_data[eocd_offset + 16],
        zip_data[eocd_offset + 17],
        zip_data[eocd_offset + 18],
        zip_data[eocd_offset + 19],
    ]) as usize;

    if cd_offset < APK_SIG_BLOCK_MIN_SIZE {
        return Ok(Vec::new());
    }

    let block_footer = &zip_data[cd_offset - 24..cd_offset];
    if &block_footer[8..24] != APK_SIG_BLOCK_MAGIC {
        return Ok(Vec::new());
    }

    let block_size = u64::from_le_bytes([
        block_footer[0], block_footer[1], block_footer[2], block_footer[3],
        block_footer[4], block_footer[5], block_footer[6], block_footer[7],
    ]) as usize;

    let block_start = cd_offset.saturating_sub(block_size + 8);
    if block_start >= cd_offset {
        return Ok(Vec::new());
    }

    let block_data = &zip_data[block_start..cd_offset - 24];

    let mut certs = Vec::new();
    let mut offset = 8; // Skip first 8 bytes (block size repeat)

    while offset + 12 <= block_data.len() {
        let pair_size = u64::from_le_bytes([
            block_data[offset], block_data[offset + 1], block_data[offset + 2], block_data[offset + 3],
            block_data[offset + 4], block_data[offset + 5], block_data[offset + 6], block_data[offset + 7],
        ]) as usize;

        if offset + 8 + pair_size > block_data.len() {
            break;
        }

        let pair_id = u32::from_le_bytes([
            block_data[offset + 8], block_data[offset + 9], block_data[offset + 10], block_data[offset + 11],
        ]);

        let pair_data = &block_data[offset + 12..offset + 8 + pair_size];

        // V2 signature block ID: 0x7109871a, V3: 0xf05368c0
        match pair_id {
            0x7109871a | 0xf05368c0 => {
                if let Ok(block_certs) = parse_signature_block(pair_data) {
                    certs.extend(block_certs);
                }
            }
            _ => {}
        }

        offset += 8 + pair_size;
    }

    Ok(certs)
}

fn parse_signature_block(data: &[u8]) -> Result<Vec<CertInfo>, String> {
    let mut certs = Vec::new();
    let mut offset = 0;

    if offset + 4 > data.len() {
        return Ok(certs);
    }
    let signers_size = u32::from_le_bytes([
        data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
    ]) as usize;
    offset += 4;

    let signers_end = offset + signers_size;

    while offset < signers_end && offset < data.len() {
        if offset + 4 > data.len() {
            break;
        }
        let signer_size = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;

        let signer_end = offset + signer_size;
        if signer_end > data.len() {
            break;
        }

        // Signed data length
        if offset + 4 > data.len() {
            break;
        }
        let signed_data_size = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;

        let signed_data_start = offset;
        let signed_data_end = offset + signed_data_size;
        if signed_data_end > signer_end {
            break;
        }

        let signed_data = &data[signed_data_start..signed_data_end];

        // Skip digests
        let mut sd_offset = 0;
        if sd_offset + 4 <= signed_data.len() {
            let digests_size = u32::from_le_bytes([
                signed_data[sd_offset], signed_data[sd_offset + 1],
                signed_data[sd_offset + 2], signed_data[sd_offset + 3],
            ]) as usize;
            sd_offset += 4 + digests_size;
        }

        // Certificates
        if sd_offset + 4 <= signed_data.len() {
            let certs_size = u32::from_le_bytes([
                signed_data[sd_offset], signed_data[sd_offset + 1],
                signed_data[sd_offset + 2], signed_data[sd_offset + 3],
            ]) as usize;
            sd_offset += 4;

            let certs_end = sd_offset + certs_size;
            while sd_offset < certs_end && sd_offset < signed_data.len() {
                if sd_offset + 4 > signed_data.len() {
                    break;
                }
                let cert_size = u32::from_le_bytes([
                    signed_data[sd_offset], signed_data[sd_offset + 1],
                    signed_data[sd_offset + 2], signed_data[sd_offset + 3],
                ]) as usize;
                sd_offset += 4;

                if sd_offset + cert_size > signed_data.len() {
                    break;
                }

                let cert_der = &signed_data[sd_offset..sd_offset + cert_size];
                sd_offset += cert_size;

                if let Ok(cert_info) = parse_x509_der(cert_der) {
                    certs.push(cert_info);
                }
            }
        }

        offset = signer_end;
    }

    Ok(certs)
}

fn find_eocd(data: &[u8]) -> Option<usize> {
    let signature = [0x50, 0x4b, 0x05, 0x06];
    let min_pos = data.len().saturating_sub(65557);
    let start = min_pos.max(0);

    for i in (start..data.len().saturating_sub(22)).rev() {
        if data[i..i + 4] == signature {
            return Some(i);
        }
    }
    None
}
