use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CertificateAnalysis {
    pub signers: Vec<SignerInfo>,
    pub signature_scheme: String,
    pub is_debug_certificate: bool,
    pub is_expired: bool,
    pub has_v1: bool,
    pub has_v2: bool,
    pub has_v3: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignerInfo {
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
