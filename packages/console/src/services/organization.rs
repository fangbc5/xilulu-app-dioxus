//! 组织管理 API
//!
//! 对接 ms-team /api/v1/team/organizations

use serde::{Deserialize, Serialize};
use serde::UrlQuery;

use super::client::{ApiClient, ApiError};

/// ============ DTO 定义 ============

/// 创建组织请求
#[derive(Debug, Serialize)]
pub struct CreateOrganizationRequest {
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    #[serde(rename = "type")]
    pub org_type: Option<String>,
    pub short_name: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 更新组织请求
#[derive(Debug, Serialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub short_name: Option<String>,
    #[serde(rename = "type")]
    pub org_type: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// 组织响应
#[derive(Debug, Deserialize)]
pub struct OrganizationResponse {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub short_name: Option<String>,
    #[serde(rename = "type")]
    pub org_type: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
    pub member_count: Option<i64>,
    pub sub_org_count: Option<i64>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}

/// 组织树节点
#[derive(Debug, Deserialize)]
pub struct OrganizationTreeNode {
    pub id: i64,
    pub name: String,
    pub short_name: Option<String>,
    pub logo: Option<String>,
    pub member_count: Option<i64>,
    pub children: Option<Vec<OrganizationTreeNode>>,
}

/// 组织列表查询
#[derive(Debug, Deserialize, Serialize)]
pub struct ListOrganizationsQuery {
    pub keyword: Option<String>,
    pub status: Option<i16>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub cursor: Option<u32>,
}

/// ============ API 方法 ============

impl ApiClient {
    /// 获取组织列表
    pub async fn list_organizations(
        &self,
        query: &ListOrganizationsQuery,
    ) -> Result<Vec<OrganizationResponse>, ApiError> {
        let query_str = query.to_query_string();
        let path = if query_str.is_empty() {
            "/api/v1/team/organizations".to_string()
        } else {
            format!("/api/v1/team/organizations?{}", query_str)
        };

        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<OrganizationResponse>>,
            pub list: Option<Vec<OrganizationResponse>>,
        }

        let resp: Resp = self.get(&path).await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 获取组织树
    pub async fn get_organization_tree(&self) -> Result<Vec<OrganizationTreeNode>, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<OrganizationTreeNode>>,
            pub list: Option<Vec<OrganizationTreeNode>>,
        }

        let resp: Resp = self.get("/api/v1/team/organizations/tree").await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 获取单个组织详情
    pub async fn get_organization(&self, id: i64) -> Result<OrganizationResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<OrganizationResponse>,
        }

        let resp: Resp = self.get(&format!("/api/v1/team/organizations/{}", id)).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 404,
            message: "Organization not found".to_string(),
        })
    }

    /// 创建组织
    pub async fn create_organization(
        &self,
        req: &CreateOrganizationRequest,
    ) -> Result<i64, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<i64>,
        }

        let resp: Resp = self.post("/api/v1/team/organizations", req).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 更新组织
    pub async fn update_organization(
        &self,
        id: i64,
        req: &UpdateOrganizationRequest,
    ) -> Result<(), ApiError> {
        self.put(&format!("/api/v1/team/organizations/{}", id), req)
            .await
    }

    /// 删除组织
    pub async fn delete_organization(&self, id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/api/v1/team/organizations/{}", id))
            .await
    }
}

/// 为查询参数实现简单的 URL 编码
trait UrlQuery {
    fn to_query_string(&self) -> String;
}

impl UrlQuery for ListOrganizationsQuery {
    fn to_query_string(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref v) = self.keyword {
            if !v.is_empty() {
                parts.push(format!("keyword={}", url_encode(v)));
            }
        }
        if let Some(v) = self.status {
            parts.push(format!("status={}", v));
        }
        if let Some(v) = self.page_size {
            parts.push(format!("pageSize={}", v));
        }
        if let Some(v) = self.cursor {
            parts.push(format!("cursor={}", v));
        }

        parts.join("&")
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
