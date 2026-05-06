use reqwest::multipart::{Form, Part};
use std::sync::OnceLock;
use std::time::Duration;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::PresignedPost;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Upload failed with status {status}: {body}")]
    UploadFailed { status: u16, body: String },
}

/// One process-wide client: a fresh `Client` per upload opens a new pool and hundreds of parallel
/// uploads can overwhelm MinIO / the kernel (connection resets mid-body). A shared client reuses
/// connections sensibly.
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            // S3-compatible POST is HTTP/1.1; avoid HTTP/2 edge cases with many large bodies.
            .http1_only()
            .pool_max_idle_per_host(128)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(600))
            .tcp_keepalive(Duration::from_secs(60))
            .build()
            .expect("reqwest::Client::build")
    })
}

pub async fn upload(post: &PresignedPost, file_path: &str) -> Result<(), Error> {
    let file = File::open(file_path).await?;
    let file_size = file.metadata().await?.len();
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file")
        .to_string();

    let stream = FramedRead::new(file, BytesCodec::new());
    let body = reqwest::Body::wrap_stream(stream);

    let mut form = Form::new();

    for (key, value) in &post.fields {
        form = form.text(key.clone(), value.clone());
    }

    let file_part = Part::stream_with_length(body, file_size).file_name(file_name);

    form = form.part("file", file_part);

    let response = http_client()
        .post(&post.url)
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await?;
        return Err(Error::UploadFailed { status, body });
    }

    Ok(())
}
