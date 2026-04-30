use crate::api::client::{ApiClient, ApiError, BASE_URL};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct PresignUploadRequest<'a> {
    pub filename: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<&'a str>,
    pub scene: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PresignUploadResponse {
    pub upload_url: String,
    pub object_key: String,
    pub file_id: i64,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct FileCallbackRequest {
    pub file_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileMetaResponse {
    pub id: i64,
    pub file_key: String,
    pub bucket: String,
    pub original_name: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<usize>,
    pub scene: String,
    pub status: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub file_id: i64,
    pub object_key: String,
    pub meta: FileMetaResponse,
}

pub async fn presign_upload(
    client: &ApiClient,
    req: PresignUploadRequest<'_>,
) -> Result<PresignUploadResponse, ApiError> {
    let url = format!("{}/api/v1/oss/presign/upload", BASE_URL);
    let mut request = client.client().post(&url).json(&req);
    request = client.inject_auth(request).await;
    client.send_request(request).await
}

pub async fn put_file_to_oss(
    upload_url: &str,
    content_type: Option<&str>,
    file_bytes: Vec<u8>,
) -> Result<(), ApiError> {
    let http_client = reqwest::Client::new();
    let mut base_req = http_client.put(upload_url);
    if let Some(ct) = content_type {
        base_req = base_req.header("Content-Type", ct);
    }
    let res = base_req
        .body(file_bytes)
        .send()
        .await
        .map_err(|e| ApiError(e.to_string()))?;

    if !res.status().is_success() {
        return Err(ApiError(format!("文件上传失败: {}", res.status())));
    }
    Ok(())
}

pub async fn oss_callback(
    client: &ApiClient,
    req: FileCallbackRequest,
) -> Result<FileMetaResponse, ApiError> {
    let url = format!("{}/api/v1/oss/callback", BASE_URL);
    let mut request = client.client().post(&url).json(&req);
    request = client.inject_auth(request).await;
    client.send_request(request).await
}

/// 封装了完整的预签名、直传、回调上传流程
pub async fn upload_file(
    client: &ApiClient,
    file_bytes: Vec<u8>,
    filename: &str,
    scene: &str,
    content_type: Option<&str>,
) -> Result<UploadResult, ApiError> {
    let size = file_bytes.len();

    // 1. 获取预签名上传 URL
    let presign_req = PresignUploadRequest {
        filename,
        content_type,
        scene,
        size: Some(size),
    };
    let presign = presign_upload(client, presign_req).await?;

    // 2. 直传文件到 OSS
    put_file_to_oss(&presign.upload_url, content_type, file_bytes).await?;

    // 3. 回调确认
    let callback_req = FileCallbackRequest {
        file_id: presign.file_id,
        size: Some(size),
    };
    let meta = oss_callback(client, callback_req).await?;

    Ok(UploadResult {
        file_id: presign.file_id,
        object_key: presign.object_key,
        meta,
    })
}
