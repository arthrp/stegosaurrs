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
mod tests {
    // use super::*;
}
