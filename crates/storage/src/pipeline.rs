//! The raster upload pipeline: validate → decode (bounded) → re-encode (strip
//! metadata) → thumbnail → checksum. Pure and CPU-bound — the caller runs it in
//! `spawn_blocking` so it never blocks the async runtime.

use bytes::Bytes;
use image::{DynamicImage, ImageError, ImageFormat, ImageReader, Limits};

use crate::StorageError;

/// Hard caps applied during decode so a small payload cannot expand into a huge
/// allocation (decompression bomb). Generous enough for any real floor/campus image.
const MAX_IMAGE_DIMENSION: u32 = 12_000;
const MAX_DECODE_ALLOC: u64 = 256 * 1024 * 1024;

/// An accepted raster format. Deliberately excludes SVG (no sanitiser) and the
/// formats with heavy/native codecs (AVIF, …).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImageKind {
    Png,
    Jpeg,
    Webp,
}

impl ImageKind {
    fn from_format(format: ImageFormat) -> Option<Self> {
        match format {
            ImageFormat::Png => Some(Self::Png),
            ImageFormat::Jpeg => Some(Self::Jpeg),
            ImageFormat::WebP => Some(Self::Webp),
            _ => None,
        }
    }

    fn to_format(self) -> ImageFormat {
        match self {
            Self::Png => ImageFormat::Png,
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Webp => ImageFormat::WebP,
        }
    }

    /// The MIME content type stored on the object and reported on fetch.
    #[must_use]
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

/// Caller-supplied size limits for an upload.
#[derive(Clone, Copy, Debug)]
pub struct ImageLimits {
    /// Maximum accepted raw byte size (rejected before decode).
    pub max_bytes: u64,
    /// Longest edge (px) of the generated thumbnail; aspect ratio preserved.
    pub thumbnail_max_px: u32,
}

/// The result of processing an upload: the re-encoded original (metadata stripped)
/// and a WebP thumbnail, with dimensions and the original's SHA-256 checksum.
#[derive(Clone, Debug)]
pub struct ProcessedUpload {
    /// The original's (preserved) format.
    pub kind: ImageKind,
    /// Re-encoded original bytes (EXIF/metadata dropped by the round-trip).
    pub original: Bytes,
    pub width: u32,
    pub height: u32,
    /// SHA-256 of [`ProcessedUpload::original`].
    pub checksum: [u8; 32],
    /// WebP thumbnail bytes.
    pub thumbnail: Bytes,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

impl ProcessedUpload {
    /// The thumbnail's content type (always WebP).
    #[must_use]
    pub fn thumbnail_content_type(&self) -> &'static str {
        "image/webp"
    }
}

/// Validates `raw` is an accepted raster image, decodes it under strict limits,
/// re-encodes it (stripping metadata) preserving format, builds a WebP thumbnail,
/// and computes the checksum.
///
/// # Errors
///
/// - [`StorageError::TooLarge`] if over `max_bytes` or the decode hits the
///   dimension/allocation limits;
/// - [`StorageError::UnsupportedFormat`] if the content is not PNG/JPEG/WebP;
/// - [`StorageError::Decode`] / [`StorageError::Encode`] on codec failure.
pub fn process_upload(raw: &[u8], limits: ImageLimits) -> Result<ProcessedUpload, StorageError> {
    if raw.len() as u64 > limits.max_bytes {
        return Err(StorageError::TooLarge);
    }

    // Magic-byte sniff: never trust the client's declared type/extension. SVG, GIF,
    // AVIF, BMP, … all resolve to a format we do not accept ⇒ rejected.
    let format = image::guess_format(raw).map_err(|_| StorageError::UnsupportedFormat)?;
    let kind = ImageKind::from_format(format).ok_or(StorageError::UnsupportedFormat)?;

    let mut reader = ImageReader::new(std::io::Cursor::new(raw));
    reader.set_format(format);
    let mut decode_limits = Limits::default();
    decode_limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    decode_limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    decode_limits.max_alloc = Some(MAX_DECODE_ALLOC);
    reader.limits(decode_limits);

    let image = match reader.decode() {
        Ok(image) => image,
        Err(ImageError::Limits(_)) => return Err(StorageError::TooLarge),
        Err(_) => return Err(StorageError::Decode),
    };
    let width = image.width();
    let height = image.height();

    let original = encode(&image, kind.to_format())?;
    let checksum = crypto::sha256(&original);

    let thumb = image.thumbnail(limits.thumbnail_max_px, limits.thumbnail_max_px);
    let thumbnail_width = thumb.width();
    let thumbnail_height = thumb.height();
    let thumbnail = encode(&thumb, ImageFormat::WebP)?;

    Ok(ProcessedUpload {
        kind,
        original: Bytes::from(original),
        width,
        height,
        checksum,
        thumbnail: Bytes::from(thumbnail),
        thumbnail_width,
        thumbnail_height,
    })
}

fn encode(image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>, StorageError> {
    let mut buf = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut buf), format)
        .map_err(|_| StorageError::Encode)?;
    Ok(buf)
}
