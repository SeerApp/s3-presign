use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;

#[derive(Debug, Clone)]
pub struct PresignedPost {
    pub url: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct PresignOptions<'a> {
    pub bucket: &'a str,
    pub bucket_url: &'a str,
    pub region: &'a str,
    pub access_key_id: &'a str,
    pub secret_access_key: &'a str,
    pub key: &'a str,
    pub max_size_bytes: u64,
    pub expires_in_seconds: i64,
}

pub fn presign(options: PresignOptions<'_>) -> PresignedPost {
    let now = Utc::now();
    let expiration = now + Duration::seconds(options.expires_in_seconds);
    let date_stamp = now.format("%Y%m%d").to_string();
    let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
    let credential = format!(
        "{}/{}/{}/s3/aws4_request",
        options.access_key_id, date_stamp, options.region
    );

    let policy = json!({
        "expiration": expiration.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "conditions": [
            {"bucket": options.bucket},
            {"key": options.key},
            ["content-length-range", 0, options.max_size_bytes],
            {"x-amz-algorithm": "AWS4-HMAC-SHA256"},
            {"x-amz-credential": credential},
            {"x-amz-date": amz_date},
        ]
    });

    let policy_b64 = STANDARD.encode(policy.to_string());
    let signature = sign_policy(
        &policy_b64,
        options.secret_access_key,
        &date_stamp,
        options.region,
    );

    let fields = vec![
        ("key".to_string(), options.key.to_string()),
        (
            "x-amz-algorithm".to_string(),
            "AWS4-HMAC-SHA256".to_string(),
        ),
        ("x-amz-credential".to_string(), credential),
        ("x-amz-date".to_string(), amz_date),
        ("policy".to_string(), policy_b64),
        ("x-amz-signature".to_string(), signature),
    ];

    PresignedPost {
        url: options.bucket_url.trim_end_matches('/').to_string(),
        fields,
    }
}

fn sign_policy(policy_b64: &str, secret_key: &str, date_stamp: &str, region: &str) -> String {
    let k_date = hmac_sha256(
        format!("AWS4{}", secret_key).as_bytes(),
        date_stamp.as_bytes(),
    );
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hmac_sha256(&k_signing, policy_b64.as_bytes());
    hex::encode(signature)
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}
