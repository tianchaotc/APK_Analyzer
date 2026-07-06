use crate::parser::ApkReader;
use crate::parser::signing;
use crate::models::certificate::*;
use std::fs;
use std::collections::HashSet;

pub struct CertificateAnalyzer;

impl super::Analyzer for CertificateAnalyzer {
    type Output = CertificateAnalysis;

    fn name(&self) -> &'static str {
        "certificate"
    }

    fn analyze(&self, apk: &mut ApkReader) -> Result<Self::Output, String> {
        let raw_apk = fs::read(&apk.file_path)
            .map_err(|e| format!("Failed to read raw APK: {}", e))?;

        let v2_certs = signing::parse_signing_block(&raw_apk).unwrap_or_default();
        let (has_v2, has_v3) = check_signature_schemes(&raw_apk);
        let v1_certs = parse_v1_signatures(apk);

        let has_v1 = !v1_certs.is_empty() && v2_certs.is_empty();

        let mut signers: Vec<SignerInfo> = Vec::new();
        let mut is_debug = false;
        let mut is_expired = false;

        let all_certs = if !v2_certs.is_empty() { v2_certs } else { v1_certs };

        for cert in &all_certs {
            let signer = SignerInfo {
                subject: cert.subject.clone(),
                issuer: cert.issuer.clone(),
                serial_number: cert.serial_number.clone(),
                sha1: cert.sha1.clone(),
                sha256: cert.sha256.clone(),
                md5: cert.md5.clone(),
                not_before: cert.not_before.clone(),
                not_after: cert.not_after.clone(),
                public_key_algorithm: cert.public_key_algorithm.clone(),
                signature_algorithm: cert.signature_algorithm.clone(),
                is_expired: cert.is_expired,
                validity_days: cert.validity_days,
            };

            if is_debug_certificate(&signer.subject) {
                is_debug = true;
            }
            if signer.is_expired {
                is_expired = true;
            }

            signers.push(signer);
        }

        let signature_scheme = if has_v3 {
            "v3 (APK Signature Scheme v3)".to_string()
        } else if has_v2 {
            "v2 (APK Signature Scheme v2)".to_string()
        } else if has_v1 {
            "v1 (JAR signing)".to_string()
        } else {
            "unsigned".to_string()
        };

        Ok(CertificateAnalysis {
            signers,
            signature_scheme,
            is_debug_certificate: is_debug,
            is_expired,
            has_v1,
            has_v2,
            has_v3,
        })
    }
}

fn check_signature_schemes(data: &[u8]) -> (bool, bool) {
    let v2_id = [0x1a, 0x87, 0x09, 0x71];
    let v3_id = [0xc0, 0x68, 0x53, 0xf0];
    (contains_bytes(data, &v2_id), contains_bytes(data, &v3_id))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn parse_v1_signatures(apk: &mut ApkReader) -> Vec<signing::CertInfo> {
    let mut certs = Vec::new();
    let file_names = apk.file_names();

    for name in &file_names {
        if name.starts_with("META-INF/") {
            let ext = name.rsplit('.').next().unwrap_or("");
            if ext == "RSA" || ext == "DSA" || ext == "EC" {
                if let Ok(data) = apk.read_file(name) {
                    certs.extend(extract_certs_from_pkcs7(&data));
                }
            }
        }
    }

    certs
}

fn extract_certs_from_pkcs7(data: &[u8]) -> Vec<signing::CertInfo> {
    let mut certs = Vec::new();
    let mut seen_sha256 = HashSet::new();
    let mut pos = 0;

    while pos + 2 < data.len() {
        if data[pos] != 0x30 {
            pos += 1;
            continue;
        }

        let Some((header_len, body_len)) = der_sequence_len(&data[pos..]) else {
            pos += 1;
            continue;
        };
        let total_len = header_len + body_len;
        if total_len <= 100 || pos + total_len > data.len() {
            pos += 1;
            continue;
        }

        let der = &data[pos..pos + total_len];
        if let Ok(cert) = signing::parse_x509_der(der) {
            if seen_sha256.insert(cert.sha256.clone()) {
                certs.push(cert);
            }
        }

        pos += 1;
    }

    certs
}

fn der_sequence_len(data: &[u8]) -> Option<(usize, usize)> {
    if data.len() < 2 || data[0] != 0x30 {
        return None;
    }

    let len_byte = data[1];
    if len_byte & 0x80 == 0 {
        return Some((2, len_byte as usize));
    }

    let len_bytes = (len_byte & 0x7f) as usize;
    if len_bytes == 0 || len_bytes > 4 || data.len() < 2 + len_bytes {
        return None;
    }

    let mut body_len = 0usize;
    for b in &data[2..2 + len_bytes] {
        body_len = (body_len << 8) | (*b as usize);
    }

    Some((2 + len_bytes, body_len))
}

fn is_debug_certificate(subject: &str) -> bool {
    subject.contains("Android Debug")
        || subject.contains("CN=Android Debug")
        || (subject.contains("O=Android") && subject.contains("CN=Android Debug"))
}
