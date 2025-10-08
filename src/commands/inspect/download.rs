use anyhow::{anyhow, Context, Result};
use aws_sdk_s3::Client;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

/// Downloads a query result file from S3 to the specified output directory
pub async fn download_from_s3(
    s3_client: &Client,
    s3_url: &str,
    output_dir: &str,
    _query_id: &str,
) -> Result<PathBuf> {
    // Parse the S3 URL to extract bucket and key
    let url = Url::parse(s3_url).context(format!("Failed to parse S3 URL: {}", s3_url))?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("Invalid S3 URL: no host in {}", s3_url))?;

    // Handle different S3 URL formats
    let (bucket, key) = if let Some(stripped) = s3_url.strip_prefix("s3://") {
        let parts: Vec<&str> = stripped.splitn(2, '/').collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid S3 URL format (s3://): {}", s3_url));
        }
        (parts[0].to_string(), parts[1].to_string())
    } else if host.ends_with(".amazonaws.com") {
        let bucket_name = host
            .split('.')
            .next()
            .ok_or_else(|| anyhow!("Invalid S3 URL: cannot extract bucket from host: {}", host))?;
        let object_key = url.path().strip_prefix('/').unwrap_or(url.path());
        (bucket_name.to_string(), object_key.to_string())
    } else {
        let path_segments = url
            .path_segments()
            .ok_or_else(|| anyhow!("Invalid S3 URL: no path in {}", s3_url))?
            .collect::<Vec<_>>();
        if path_segments.is_empty() {
            return Err(anyhow!("Invalid S3 URL: empty path in {}", s3_url));
        }
        let bucket_name = path_segments[0];
        let object_key = path_segments[1..].join("/");
        (bucket_name.to_string(), object_key)
    };

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir)
        .context(format!("Failed to create output directory: {}", output_dir))?;

    // Extract filename from the key
    let filename_from_key = Path::new(&key)
        .file_name()
        .ok_or_else(|| anyhow!("Could not extract filename from S3 key: {}", key))?
        .to_string_lossy()
        .to_string();

    // Create output file path
    let output_path = Path::new(output_dir).join(&filename_from_key);

    // Get the object from S3
    let resp = s3_client
        .get_object()
        .bucket(&bucket)
        .key(&key)
        .send()
        .await
        .context(format!(
            "Failed to download file from S3 bucket: {}, key: {}",
            bucket, key
        ))?;

    // Read the data
    let data = resp
        .body
        .collect()
        .await
        .context("Failed to read S3 object data stream")?;
    let bytes = data.into_bytes();

    // Write to file
    let mut file = File::create(&output_path).context(format!(
        "Failed to create output file: {}",
        output_path.display()
    ))?;
    file.write_all(&bytes).context(format!(
        "Failed to write data to file: {}",
        output_path.display()
    ))?;

    Ok(output_path)
}