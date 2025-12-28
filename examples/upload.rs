use s3_presign::{presign, upload, PresignOptions};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: cargo run --example upload <bucket> <file_path> [key]");
        eprintln!();
        eprintln!("Environment variables:");
        eprintln!("  S3_ENDPOINT        - S3 endpoint URL (default: http://localhost:9000)");
        eprintln!("  S3_ACCESS_KEY      - Access key (default: minioadmin)");
        eprintln!("  S3_SECRET_KEY      - Secret key (default: minioadmin)");
        eprintln!("  S3_REGION          - Region (default: us-east-1)");
        eprintln!("  S3_MAX_SIZE        - Max file size in bytes (default: 104857600 = 100MB)");
        std::process::exit(1);
    }

    let bucket = &args[1];
    let file_path = &args[2];
    let key = args.get(3).map(|s| s.as_str()).unwrap_or_else(|| {
        std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
    });

    let endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let access_key = env::var("S3_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let secret_key = env::var("S3_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let region = env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let max_size: u64 = env::var("S3_MAX_SIZE")
        .unwrap_or_else(|_| "104857600".to_string())
        .parse()
        .unwrap_or(100 * 1024 * 1024);

    let bucket_url = format!("{}/{}", endpoint.trim_end_matches('/'), bucket);

    println!("Uploading {} -> {}/{}", file_path, bucket, key);
    println!("Endpoint: {}", endpoint);
    println!("Bucket URL: {}", bucket_url);

    let post = presign(PresignOptions {
        bucket,
        bucket_url: &bucket_url,
        region: &region,
        access_key_id: &access_key,
        secret_access_key: &secret_key,
        key,
        max_size_bytes: max_size,
        expires_in_seconds: 3600,
    });

    println!("POST URL: {}", post.url);
    println!("Fields: {:?}", post.fields);

    upload(&post, file_path).await?;

    println!("Done: {}/{}", bucket_url, key);
    Ok(())
}
