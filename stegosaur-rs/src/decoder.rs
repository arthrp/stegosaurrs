use crate::misc;

pub fn decode_lossless(image_data: &[u8]) -> Result<String, String> {
    let img = image::load_from_memory(image_data).expect("Failed to decode image");
    let rgb = img.to_rgb8();
    let mut pixels = rgb.enumerate_pixels();

    // Read and verify 3 HEADER bytes from LSB of R channel of first 24 pixels
    let mut read_header = [0u8; 3];
    for byte in &mut read_header {
        for i in 0..8 {
            if let Some((_, _, pixel)) = pixels.next() {
                let bit = pixel[0] & 1;
                *byte |= bit << i;
            }
        }
    }
    if read_header != misc::HEADER {
        return Err("Image does not contain steganographic data (invalid header)".to_string());
    }

    // Read text size (little-endian u16) from LSB of R channel of next 16 pixels
    let mut size: u16 = 0;
    for i in 0..16 {
        if let Some((_, _, pixel)) = pixels.next() {
            let bit = (pixel[0] & 1) as u16;
            size |= bit << i;
        }
    }

    // Read text bytes (LSB-first) from LSB of R channel of subsequent pixels
    let mut bytes = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let mut byte: u8 = 0;
        for i in 0..8 {
            if let Some((_, _, pixel)) = pixels.next() {
                let bit = pixel[0] & 1;
                byte |= bit << i;
            }
        }
        bytes.push(byte);
    }

    String::from_utf8(bytes)
        .map_err(|_| "Decoded text is not valid UTF-8".to_string())
}