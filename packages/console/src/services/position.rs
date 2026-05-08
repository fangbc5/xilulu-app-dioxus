//! 岗位管理 API
//!
//! 对接 ms-team /api/v1/team/positions

use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

/// ============ DTO 定义 ============

/// 创建岗位请求
#[derive(Debug, Serialize)]
pub struct CreatePositionRequest {
    #[serde(rename = "orgId")]
    pub org_id: i64,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub requirements: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
}

/// 更新岗位请求
#[derive(Debug, Serialize)]
pub struct UpdatePositionRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub requirements: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// 岗位响应
#[derive(Debug, Deserialize)]
pub struct PositionResponse {
    pub id: i64,
    #[serde(rename = "orgId")]
    pub org_id: i64,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub level: Option<i32>,
    pub description: Option<String>,
    pub requirements: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
    #[serde(rename = "employeeCount")]
    pub employee_count: Option<i64>,
    #[serde(rename = "createTime")]
    pub create_time: Option<String>,
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
}

/// 岗位列表查询
#[derive(Debug, Serialize)]
pub struct ListPositionsQuery {
    #[serde(rename = "orgId")]
    pub org_id: Option<i64>,
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub status: Option<i16>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub cursor: Option<u32>,
}

/// ============ API 方法 ============

impl ApiClient {
    /// 获取岗位列表
    ///
    /// 服务端返回 `R<CursorPageBaseResp<PositionResponse>>`，
    /// 即 `{ data: { cursor, has_next, list: [...], total } }`。
    pub async fn list_positions(
        &self,
        query: &ListPositionsQuery,
    ) -> Result<Vec<PositionResponse>, ApiError> {
        let query_str = build_position_query(query);
        let path = format!("/api/v1/team/positions{}", query_str);

        #[derive(Debug, Deserialize)]
        struct Page {
            list: Vec<PositionResponse>,
        }
        #[derive(Debug, Deserialize)]
        struct Resp {
            data: Option<Page>,
        }

        let resp: Resp = self.get(&path).await?;
        Ok(resp.data.map(|p| p.list).unwrap_or_default())
    }

    /// 统计岗位数量（page_size=1，从分页响应的 total 字段获取总数）
    pub async fn count_positions(&self, org_id: i64) -> Result<u32, ApiError> {
        let path = format!(
            "/api/v1/team/positions?org_id={}&page_size=1&cursor=1",
            org_id
        );

        #[derive(Debug, Deserialize)]
        struct Page {
            total: Option<i64>,
        }
        #[derive(Debug, Deserialize)]
        struct Resp {
            data: Option<Page>,
        }

        let resp: Resp = self.get(&path).await?;
        Ok(resp.data.and_then(|d| d.total).unwrap_or(0) as u32)
    }

    /// 获取单个岗位详情
    pub async fn get_position(&self, id: i64) -> Result<PositionResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<PositionResponse>,
        }

        let resp: Resp = self.get(&format!("/api/v1/team/positions/{}", id)).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 404,
            message: "Position not found".to_string(),
        })
    }

    /// 创建岗位
    pub async fn create_position(&self, req: &CreatePositionRequest) -> Result<i64, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<i64>,
        }

        let resp: Resp = self.post("/api/v1/team/positions", req).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 更新岗位
    pub async fn update_position(
        &self,
        id: i64,
        req: &UpdatePositionRequest,
    ) -> Result<(), ApiError> {
        self.put(&format!("/api/v1/team/positions/{}", id), req)
            .await
    }

    /// 删除岗位
    pub async fn delete_position(&self, id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/api/v1/team/positions/{}", id)).await
    }
}

/// 辅助函数：构建查询字符串
fn build_position_query(query: &ListPositionsQuery) -> String {
    let mut parts = Vec::new();

    if let Some(v) = query.org_id {
        parts.push(format!("org_id={}", v));
    }
    if let Some(ref v) = query.category {
        if !v.is_empty() {
            parts.push(format!("category={}", url_encode(v)));
        }
    }
    if let Some(ref v) = query.keyword {
        if !v.is_empty() {
            parts.push(format!("keyword={}", url_encode(v)));
        }
    }
    if let Some(v) = query.status {
        parts.push(format!("status={}", v));
    }
    if let Some(v) = query.page_size {
        parts.push(format!("page_size={}", v));
    }
    if let Some(v) = query.cursor {
        parts.push(format!("cursor={}", v));
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    result
}
