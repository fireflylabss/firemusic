use std::process::Command;

pub fn extract_cover_art(path: &str) -> Option<Vec<u8>> {
    let output = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            path,
            "-an",
            "-vcodec",
            "copy",
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "pipe:1",
        ])
        .output()
        .ok()?;
    if output.status.success() && output.stdout.len() > 8 {
        Some(output.stdout)
    } else {
        None
    }
}

pub fn png_dimensions(data: &[u8]) -> (u32, u32) {
    if data.len() >= 24 && &data[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
        (w, h)
    } else {
        (0, 0)
    }
}