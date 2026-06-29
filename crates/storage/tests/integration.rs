//! Storage integration tests.
//!
//! Pipeline tests (validation, re-encode, rejection) need no services. The S3
//! round-trip + presigned-URL tests talk to a real SeaweedFS S3 gateway — the dev
//! stack (`deploy/dev/compose.yaml`) or the CI-started container — defaulting to
//! `localhost:8333` with the dev identity, overridable via `OWK_S3_*` env.

use std::io::Cursor;

use config::StorageConfig;
use image::{DynamicImage, ImageFormat};
use secrecy::SecretString;
use storage::{ImageLimits, Storage, new_storage_key, process_upload, thumbnail_key};

fn limits() -> ImageLimits {
    ImageLimits {
        max_bytes: 10 * 1024 * 1024,
        thumbnail_max_px: 64,
    }
}

fn sample(width: u32, height: u32, format: ImageFormat) -> Vec<u8> {
    let img = DynamicImage::new_rgb8(width, height);
    let mut buf = Vec::new();
    img.write_to(&mut Cursor::new(&mut buf), format).unwrap();
    buf
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn s3_config() -> StorageConfig {
    StorageConfig {
        kind: "s3".to_owned(),
        s3_endpoint: env_or("OWK_S3_ENDPOINT", "http://localhost:8333"),
        s3_region: "us-east-1".to_owned(),
        s3_bucket: env_or("OWK_S3_BUCKET", "openworkspace"),
        s3_access_key: SecretString::from(env_or("OWK_S3_ACCESS_KEY", "openworkspacedev")),
        s3_secret_key: SecretString::from(env_or("OWK_S3_SECRET_KEY", "openworkspacedevsecret")),
        s3_allow_http: true,
        s3_force_path_style: true,
        ..StorageConfig::default()
    }
}

// --- pipeline (no services) --------------------------------------------------

#[test]
fn accepts_png_jpeg_webp_and_thumbnails() {
    for format in [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::WebP] {
        let raw = sample(200, 100, format);
        let out = process_upload(&raw, limits()).unwrap();
        assert_eq!(out.width, 200);
        assert_eq!(out.height, 100);
        assert!(!out.original.is_empty());
        assert_ne!(out.checksum, [0u8; 32]);
        // Thumbnail fits the 64px box and preserves aspect ratio (longest edge 64).
        assert!(out.thumbnail_width <= 64 && out.thumbnail_height <= 64);
        assert_eq!(out.thumbnail_width, 64, "longest edge scaled to the cap");
        assert!(!out.thumbnail.is_empty());
    }
}

#[test]
fn rejects_non_raster_and_unaccepted_formats() {
    // SVG: not a raster image at all.
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect/></svg>"#;
    assert!(matches!(
        process_upload(svg, limits()),
        Err(storage::StorageError::UnsupportedFormat)
    ));
    // GIF: a real raster format (detected by magic bytes) we deliberately reject.
    let gif = b"GIF89a\x01\x00\x01\x00\x00\x00\x00;";
    assert!(matches!(
        process_upload(gif, limits()),
        Err(storage::StorageError::UnsupportedFormat)
    ));
    // Random bytes: undetectable format.
    assert!(matches!(
        process_upload(b"not an image at all", limits()),
        Err(storage::StorageError::UnsupportedFormat)
    ));
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// A real JPEG with a `COM` (comment) segment carrying `sentinel` spliced in after
/// the SOI marker — a stand-in for the EXIF/metadata a camera embeds. Re-encoding
/// must drop it (the same mechanism that strips EXIF/GPS).
fn jpeg_with_metadata(sentinel: &[u8]) -> Vec<u8> {
    let base = sample(64, 48, ImageFormat::Jpeg); // starts with SOI (0xFF 0xD8)
    let seg_len = u16::try_from(sentinel.len() + 2).unwrap();
    let mut out = Vec::with_capacity(base.len() + sentinel.len() + 4);
    out.extend_from_slice(&base[..2]); // SOI
    out.extend_from_slice(&[0xFF, 0xFE]); // COM marker
    out.extend_from_slice(&seg_len.to_be_bytes());
    out.extend_from_slice(sentinel);
    out.extend_from_slice(&base[2..]);
    out
}

#[test]
fn reencode_strips_embedded_metadata() {
    let sentinel = b"OWK-PRIVATE-EXIF-GPS-SENTINEL";
    let raw = jpeg_with_metadata(sentinel);
    assert!(contains(&raw, sentinel), "fixture must embed the sentinel");
    let out = process_upload(&raw, limits()).unwrap();
    assert!(
        !contains(&out.original, sentinel),
        "re-encoding must strip embedded metadata"
    );
}

#[test]
fn thumbnail_downscales_large_image_preserving_aspect() {
    let raw = sample(1600, 1200, ImageFormat::Jpeg);
    let out = process_upload(&raw, limits()).unwrap();
    assert_eq!((out.width, out.height), (1600, 1200));
    // 4:3 scaled so the longest edge hits the 64px cap.
    assert_eq!((out.thumbnail_width, out.thumbnail_height), (64, 48));
}

#[test]
fn rejects_oversized_before_decode() {
    let raw = sample(64, 64, ImageFormat::Png);
    let tight = ImageLimits {
        max_bytes: 4,
        thumbnail_max_px: 64,
    };
    assert!(matches!(
        process_upload(&raw, tight),
        Err(storage::StorageError::TooLarge)
    ));
}

// --- S3 round-trip (needs SeaweedFS) -----------------------------------------

#[tokio::test]
async fn s3_put_presign_fetch_round_trip() {
    // The test's reqwest (rustls, no bundled provider) needs aws-lc-rs installed
    // process-wide, matching the app's single crypto provider (no ring).
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
    let storage = Storage::from_config(&s3_config()).unwrap();
    let raw = sample(120, 80, ImageFormat::Png);
    let processed = process_upload(&raw, limits()).unwrap();

    let key = format!("test/{}", new_storage_key());
    let thumb = thumbnail_key(&key);
    storage
        .put(
            &key,
            processed.original.clone(),
            processed.kind.content_type(),
        )
        .await
        .unwrap();
    storage
        .put(
            &thumb,
            processed.thumbnail.clone(),
            processed.thumbnail_content_type(),
        )
        .await
        .unwrap();

    // Direct get round-trips the exact bytes.
    let fetched = storage.get(&key).await.unwrap();
    assert_eq!(fetched, processed.original);

    // Presigned GET URL is fetchable by an unauthenticated HTTP client.
    let url = storage.signed_get_url(&key).await.unwrap();
    let resp = reqwest::Client::new().get(&url).send().await.unwrap();
    assert!(
        resp.status().is_success(),
        "presigned GET failed: {}",
        resp.status()
    );
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let body = resp.bytes().await.unwrap();
    assert_eq!(body, processed.original, "presigned body must match");
    assert_eq!(
        content_type, "image/png",
        "stored content-type must be served"
    );

    // Cleanup.
    storage.delete(&key).await.unwrap();
    storage.delete(&thumb).await.unwrap();
}
