//! `xwww screenshot` — captures the wallpaper the daemon is currently displaying on an output and
//! saves it as a PNG file.
//!
//! The daemon reads its own canvas buffer, normalizes it to RGB, and sends it back over the IPC
//! socket. This client only has to re-encode those bytes into a PNG.

use common::ipc::{Answer, IpcSocket, RequestSend, ScreenshotReq};

use crate::cli::Screenshot;

pub fn run(screenshot: &Screenshot) -> Result<(), String> {
    let namespace = screenshot
        .namespace
        .first()
        .map(String::as_str)
        .unwrap_or("");
    let output = screenshot.monitor.clone().unwrap_or_default();

    let socket = IpcSocket::client(namespace).map_err(|e| e.to_string())?;

    let request = ScreenshotReq {
        output: output.into_boxed_str(),
    };
    let mmap = request.create_request().map_err(|e| e.to_string())?;
    RequestSend::Screenshot(mmap)
        .send(&socket)
        .map_err(|e| e.to_string())?;

    let bytes = socket.recv().map_err(|e| e.to_string())?;
    let Answer::Screenshot(data) = Answer::receive(bytes) else {
        return Err("Daemon did not return Screenshot, as expected".to_string());
    };

    if data.width == 0 || data.height == 0 {
        return Err("no wallpaper to capture (is the daemon running and initialized?)".to_string());
    }

    let img = image::RgbImage::from_raw(data.width, data.height, data.rgb.into_vec())
        .ok_or_else(|| "failed to build image from screenshot data".to_string())?;
    img.save(&screenshot.output)
        .map_err(|e| format!("failed to save screenshot: {e}"))?;

    Ok(())
}
