#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::WebViewHostWindows;

#[cfg(not(windows))]
pub use windows::WebViewHostWindows;
