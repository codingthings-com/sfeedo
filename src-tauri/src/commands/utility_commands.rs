use std::process::Command;

#[tauri::command]
pub async fn open_url_in_browser(url: String) -> Result<(), String> {
    // Validate URL
    if url.is_empty() {
        return Err("URL cannot be empty".to_string());
    }

    // Ensure URL has protocol
    let valid_url = if !url.starts_with("http://") && !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url
    };

    // Use system command to open URL in default browser
    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(&valid_url).spawn();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(&valid_url).spawn();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd")
        .args(&["/C", "start", &valid_url])
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to open URL: {}", e)),
    }
}
