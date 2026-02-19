use image::{ImageBuffer, Rgb, error::{ParameterError, ParameterErrorKind}};

use crate::misc;

const MAX_TEXT_LENGTH: usize = 65535;

pub fn encode_lossless(
    text: &str,
    image_data: &[u8]
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, image::ImageError> {
    if text.len() > MAX_TEXT_LENGTH {
        return Err(image::ImageError::Parameter(
            ParameterError::from_kind(ParameterErrorKind::Generic(
                format!("Maximum text length that can be encoded is {}", MAX_TEXT_LENGTH),
            )),
        ));
    }
    

    if !text.is_ascii() {
        return Err(image::ImageError::Parameter(
            ParameterError::from_kind(ParameterErrorKind::Generic(
                format!("Only ASCII characters are supported"),
            )),
        ));
    }

    let img = image::load_from_memory(image_data).expect("Failed to decode image");
    let total_pixels = img.width() * img.height();

    if total_pixels < ((text.len() * 8) + 16 + 3*8) as u32 {
        return Err(image::ImageError::Parameter(
            ParameterError::from_kind(ParameterErrorKind::Generic(
                format!("Image size {} cannot fit text size of {}", total_pixels, text.len()),
            )),
        ));
    }

    let mut rgb = img.to_rgb8();

    let size = text.len() as u16;
    let text_bytes = text.as_bytes();
    let mut pixels = rgb.enumerate_pixels_mut();

    // Embed 3 HEADER bytes in LSB of R channel of first 24 pixels
    for &byte in &misc::HEADER {
        for i in 0..8 {
            if let Some((_, _, pixel)) = pixels.next() {
                let bit = (byte >> i) & 1;
                pixel[0] = (pixel[0] & 0xFE) | bit;
            }
        }
    }

    // Embed text size (little-endian u16) in LSB of R channel of next 16 pixels
    for i in 0..16 {
        if let Some((_, _, pixel)) = pixels.next() {
            let bit = ((size >> i) & 1) as u8;
            pixel[0] = (pixel[0] & 0xFE) | bit;
        }
    }

    // Embed text bytes (LSB-first) in LSB of R channel of subsequent pixels
    for byte in text_bytes {
        for i in 0..8 {
            if let Some((_, _, pixel)) = pixels.next() {
                let bit = (byte >> i) & 1;
                pixel[0] = (pixel[0] & 0xFE) | bit;
            }
        }
    }

    Ok(rgb)
}

#[cfg(test)]
mod tests {
    use image::codecs::png::PngEncoder;
    use image::ExtendedColorType;
    use image::ImageEncoder;

    use super::*;

    fn create_test_image(width: u32, height: u32) -> Vec<u8> {
        let img = image::RgbImage::new(width, height);
        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        encoder
            .write_image(img.as_raw(), width, height, ExtendedColorType::Rgb8)
            .unwrap();
        bytes
    }

    fn encode_rgb_to_png_bytes(rgb: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Vec<u8> {
        let (width, height) = rgb.dimensions();
        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        encoder
            .write_image(rgb.as_raw(), width, height, ExtendedColorType::Rgb8)
            .unwrap();
        bytes
    }

    #[test]
    fn test_text_too_long() {
        let long_text = "x".repeat(65536);
        let image_data = create_test_image(100, 100);

        let result = encode_lossless(&long_text, &image_data);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Maximum text length"));
    }

    #[test]
    fn test_non_ascii_rejected() {
        let image_data = create_test_image(10, 10);

        for non_ascii in ["café", "日本語", "café au lait"] {
            let result = encode_lossless(non_ascii, &image_data);
            assert!(result.is_err(), "Expected error for non-ASCII: {}", non_ascii);
            let err = result.unwrap_err();
            assert!(err.to_string().contains("Only ASCII"));
        }
    }

    #[test]
    fn test_image_too_small() {
        // "x" needs (1*8) + 16 + 24 = 48 pixels minimum. 1x1 = 1 pixel.
        let image_data = create_test_image(1, 1);

        let result = encode_lossless("x", &image_data);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cannot fit text size"));
    }

    #[test]
    fn test_encode_preserves_image_dimensions() {
        let width = 10u32;
        let height = 10u32;
        let image_data = create_test_image(width, height);

        let rgb = encode_lossless("Hi", &image_data).unwrap();
        let output_bytes = encode_rgb_to_png_bytes(&rgb);
        let output_img = image::load_from_memory(&output_bytes).unwrap();
        assert_eq!(output_img.width(), width);
        assert_eq!(output_img.height(), height);
    }
}