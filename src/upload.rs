use reqwest::multipart::{Form, Part};
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

pub async fn upload(client: &reqwest::Client, post: &PresignedPost, file_path: &str) -> Result<(), Error> {
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

    let response = client
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
