use aws_sdk_s3::config::{Credentials, Region};
use s3_presign::{presign, upload, Error, PresignOptions};
use std::io::Write;
use tempfile::NamedTempFile;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::minio::MinIO;

const ACCESS_KEY: &str = "minioadmin";
const SECRET_KEY: &str = "minioadmin";
const REGION: &str = "us-east-1";

async fn create_bucket(endpoint: &str, bucket: &str) {
    let creds = Credentials::new(ACCESS_KEY, SECRET_KEY, None, None, "test");
    let config = aws_sdk_s3::Config::builder()
        .endpoint_url(endpoint)
        .region(Region::new(REGION))
        .credentials_provider(creds)
        .force_path_style(true)
        .behavior_version_latest()
        .build();

    let client = aws_sdk_s3::Client::from_conf(config);
    client.create_bucket().bucket(bucket).send().await.unwrap();
}

#[tokio::test]
async fn test_upload_success() {
    let container = MinIO::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(9000).await.unwrap();
    let endpoint = format!("http://127.0.0.1:{}", port);
    let bucket = "test-bucket-success";

    create_bucket(&endpoint, bucket).await;

    // Create a small temp file (100 bytes)
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(&[b'a'; 100]).unwrap();
    temp_file.flush().unwrap();

    let post = presign(PresignOptions {
        bucket,
        bucket_url: &format!("{}/{}", endpoint, bucket),
        region: REGION,
        access_key_id: ACCESS_KEY,
        secret_access_key: SECRET_KEY,
        key: "test-file.txt",
        max_size_bytes: 1024, // Allow up to 1KB
        expires_in_seconds: 3600,
    });

    let result = upload(&post, temp_file.path().to_str().unwrap()).await;
    assert!(result.is_ok(), "Upload should succeed: {:?}", result);
}

#[tokio::test]
async fn test_upload_file_too_large() {
    let container = MinIO::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(9000).await.unwrap();
    let endpoint = format!("http://127.0.0.1:{}", port);
    let bucket = "test-bucket-large";

    create_bucket(&endpoint, bucket).await;

    // Create a file larger than the limit (200 bytes)
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(&[b'a'; 200]).unwrap();
    temp_file.flush().unwrap();

    let post = presign(PresignOptions {
        bucket,
        bucket_url: &format!("{}/{}", endpoint, bucket),
        region: REGION,
        access_key_id: ACCESS_KEY,
        secret_access_key: SECRET_KEY,
        key: "test-file-large.txt",
        max_size_bytes: 100, // Only allow 100 bytes
        expires_in_seconds: 3600,
    });

    let result = upload(&post, temp_file.path().to_str().unwrap()).await;

    match result {
        Err(Error::UploadFailed { status, .. }) => {
            assert_eq!(status, 400, "Expected 400 Bad Request for oversized file");
        }
        Ok(_) => panic!("Upload should have failed for oversized file"),
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}
