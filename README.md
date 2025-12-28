# s3-presign

Tiny SDK for S3 presigned uploads: 
- `presign(...)` to generate POST URLs with server-side file size limits.
- `upload(...)` to stream files to the signed URL.

## Installation

```toml
[dependencies]
s3-presign = { git = "https://github.com/SeerApp/s3-presign", tag = "v0.1.0" }
```

## Usage

```rust
use s3_presign::{presign, upload, PresignOptions};

// Generate presigned POST
let post = presign(PresignOptions {
    bucket: "my-bucket",
    bucket_url: "https://s3.amazonaws.com/my-bucket",
    region: "us-east-1",
    access_key_id: "AKIAIOSFODNN7EXAMPLE",
    secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    key: "uploads/file.txt",
    max_size_bytes: 10 * 1024 * 1024, // 10MB limit
    expires_in_seconds: 3600,
});

// Upload file
upload(&post, "/path/to/file.txt").await?;
```
