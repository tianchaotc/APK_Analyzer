use crate::parser::ApkReader;
use crate::parser::signing;
use crate::models::certificate::*;
use std::fs;

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
                    if let Ok(cert_der) = extract_cert_from_pkcs7(&data) {
                        if let Ok(cert) = signing::parse_x509_der(&cert_der) {
                            certs.push(cert);
                        }
                    }
                }
            }
        }
    }

    certs
}

fn extract_cert_from_pkcs7(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut pos = 0;
    while pos < data.len().saturating_sub(4) {
        if data[pos] == 0x30 && data[pos + 1] == 0x82 {
            let cert_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
            if cert_len > 100 && pos + 4 + cert_len <= data.len() {
                return Ok(data[pos..pos + 4 + cert_len].to_vec());
            }
        }
        pos += 1;
    }
    Err("No certificate found in PKCS#7".to_string())
}

fn is_debug_certificate(subject: &str) -> bool {
    subject.contains("Android Debug")
        || subject.contains("CN=Android Debug")
        || (subject.contains("O=Android") && subject.contains("CN=Android Debug"))
}
