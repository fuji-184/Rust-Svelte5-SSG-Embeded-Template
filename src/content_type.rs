pub fn get_content_type(path: &str) -> String {
    let ext = path.rsplit(".").next().unwrap_or_default().to_lowercase();
    match ext.as_str() {
        "html" => "content-type: text/html".to_string(),
        "css" => "content-type: text/css".to_string(),
        "js" => "content-type: application/javascript".to_string(),
        "json" => "content-type: application/json".to_string(),
        "jpg" | "jpeg" => "content-type: image/jpeg".to_string(),
        "png" => "content-type: image/png".to_string(),
        "gif" => "content-type: image/gif".to_string(),
        "svg" => "content-type: image/svg+xml".to_string(),
        "txt" => "content-type: text/plain".to_string(),
        _ => "content-type: application/octet-stream".to_string(),
    }
}

