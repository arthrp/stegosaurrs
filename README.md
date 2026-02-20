# Stegosaur-rs

<p align="center">
    <picture>
        <img src="logo.png" alt="logo" >
    </picture>
</p>

Stegosaur-rs is a steganography library and cli tool that allows to embed hidden text inside images. When embedding into lossless format (png), it embeds one bit of information per pixel, using LSB of red color value which produces nearly identical image to the original.
