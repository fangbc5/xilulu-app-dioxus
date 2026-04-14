use crate::api::GLOBAL_API_CLIENT;

pub async fn core_upload_file(
    file_bytes: Vec<u8>,
    filename: String,
    scene: String,
) -> Result<String, String> {
    let content_type = if filename.ends_with(".png") {
        Some("image/png")
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if filename.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    };

    use xilulu_core::api::oss::upload_file;
    match upload_file(
        &GLOBAL_API_CLIENT,
        file_bytes,
        &filename,
        &scene,
        content_type,
    )
    .await
    {
        Ok(res) => serde_json::to_string(&res.meta).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
