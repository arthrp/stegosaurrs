# Stegosaur-rs

<p align="center">
    <picture>
        <img src="logo.png" alt="logo" >
    </picture>
</p>

Stegosaur-rs is a steganography library and cli tool that allows to embed hidden text inside images. When embedding into lossless format (png), it embeds one bit of information per pixel, using LSB of red color value which produces nearly identical image to the original.

## Hidden Data Format

| Field | Size | Pixels | Description |
|-------|------|--------|-------------|
| Header | 3 bytes | 24 | Magic bytes `0x53 0x54 0x47` ("STG") to identify steganographic data |
| Text size | 2 bytes | 16 | Little-endian u16: number of payload bytes (max 65535) |
| Payload | variable | size × 8 | ASCII text bytes, LSB-first per byte |

**Minimum pixels required:** 40 + (text length × 8)
