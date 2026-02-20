use std::{fs, process};

mod encoder;
mod decoder;
mod misc;

pub fn encode_lossless(file_path: &str, text: &str, output_path: &str) -> Result<(), image::ImageError> {
    let image_data = fs::read(file_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", file_path, err);
        process::exit(1);
    });

    let rgb = encoder::encode_lossless(text, &image_data)?;
    rgb.save(output_path)?;
    Ok(())
}

pub fn decode_lossless(file_path: &str) -> Result<String, String> {
    let image_data = fs::read(file_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", file_path, err);
        process::exit(1);
    });

    decoder::decode_lossless(&image_data)
}
#[cfg(test)]
mod lib_tests {
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};

    use super::*;

    fn get_output_filepath() -> PathBuf {
        let tmp = std::env::temp_dir();
        let time = SystemTime::now();
        let unix_time = time.duration_since(UNIX_EPOCH).expect("Can't get time");

        tmp.join(format!("test_{}.png", unix_time.as_secs()))
    }

    fn create_source_image_file(width: u32, height: u32) -> PathBuf {
        let img = image::RgbImage::new(width, height);
        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        encoder
            .write_image(img.as_raw(), width, height, ExtendedColorType::Rgb8)
            .unwrap();
        let path = std::env::temp_dir().join(format!(
            "source_{}.png",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        ));
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn test_round_trip_encode_decode() {
        let source = create_source_image_file(100, 100);
        let output = get_output_filepath();
        encode_lossless(
            source.to_str().unwrap(),
            "Hello, world!",
            output.to_str().unwrap(),
        )
        .unwrap();
        let decoded = decode_lossless(output.to_str().unwrap()).unwrap();
        assert_eq!(decoded, "Hello, world!");
    }

    #[test]
    fn test_decode_plain_image_fails() {
        let source = create_source_image_file(100, 100);
        let result = decode_lossless(source.to_str().unwrap());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("invalid header"));
    }

    #[test]
    fn test_encode_small_image_returns_error() {
        let source = create_source_image_file(1, 1);
        let output = get_output_filepath();
        let result = encode_lossless(source.to_str().unwrap(), "x", output.to_str().unwrap());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("cannot fit text size"));
    }
}
