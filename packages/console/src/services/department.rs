//! 部门管理 API
//!
//! 对接 ms-team /api/v1/team/departments

use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

/// ============ DTO 定义 ============

/// 创建部门请求
#[derive(Debug, Serialize)]
pub struct CreateDepartmentRequest {
    #[serde(rename = "orgId")]
    pub org_id: i64,
    #[serde(rename = "parentId")]
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    #[serde(rename = "leaderEmployeeId")]
    pub leader_employee_id: Option<i64>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
}

/// 更新部门请求
#[derive(Debug, Serialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    #[serde(rename = "leaderEmployeeId")]
    pub leader_employee_id: Option<i64>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// 部门响应
#[derive(Debug, Deserialize)]
pub struct DepartmentResponse {
    pub id: i64,
    #[serde(rename = "orgId")]
    pub org_id: i64,
    #[serde(rename = "parentId")]
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    pub path: Option<String>,
    #[serde(rename = "leaderEmployeeId")]
    pub leader_employee_id: Option<i64>,
    #[serde(rename = "leaderName")]
    pub leader_name: Option<String>,
    pub level: Option<i32>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
    #[serde(rename = "employeeCount")]
    pub employee_count: Option<i64>,
    #[serde(rename = "subDeptCount")]
    pub sub_dept_count: Option<i64>,
    #[serde(rename = "createTime")]
    pub create_time: Option<String>,
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
}

/// 部门树节点
#[derive(Debug, Deserialize)]
pub struct DepartmentTreeNode {
    pub id: i64,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "employeeCount")]
    pub employee_count: Option<i64>,
    #[serde(rename = "leaderName")]
    pub leader_name: Option<String>,
    pub children: Option<Vec<DepartmentTreeNode>>,
}

/// 部门列表查询
#[derive(Debug, Serialize)]
pub struct ListDepartmentsQuery {
    #[serde(rename = "orgId")]
    pub org_id: Option<i64>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<i64>,
    pub keyword: Option<String>,
    pub status: Option<i16>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub cursor: Option<u32>,
}

/// ============ API 方法 ============

impl ApiClient {
    /// 获取部门列表
    pub async fn list_departments(
        &self,
        query: &ListDepartmentsQuery,
    ) -> Result<Vec<DepartmentResponse>, ApiError> {
        let query_str = build_query(query);
        let path = format!("/api/v1/team/departments{}", query_str);

        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<DepartmentResponse>>,
            pub list: Option<Vec<DepartmentResponse>>,
        }

        let resp: Resp = self.get(&path).await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 获取根部门列表
    pub async fn get_root_departments(&self, org_id: i64) -> Result<Vec<DepartmentResponse>, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<DepartmentResponse>>,
            pub list: Option<Vec<DepartmentResponse>>,
        }

        let resp: Resp = self
            .get(&format!(
                "/api/v1/team/departments/roots?orgId={}",
                org_id
            ))
            .await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 获取部门树
    pub async fn get_department_tree(
        &self,
        org_id: i64,
    ) -> Result<Vec<DepartmentTreeNode>, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<DepartmentTreeNode>>,
            pub list: Option<Vec<DepartmentTreeNode>>,
        }

        let resp: Resp = self
            .get(&format!(
                "/api/v1/team/departments/tree?orgId={}",
                org_id
            ))
            .await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 获取单个部门详情
    pub async fn get_department(&self, id: i64) -> Result<DepartmentResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<DepartmentResponse>,
        }

        let resp: Resp = self.get(&format!("/api/v1/team/departments/{}", id)).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 404,
            message: "Department not found".to_string(),
        })
    }

    /// 获取子部门
    pub async fn get_children_departments(
        &self,
        id: i64,
    ) -> Result<Vec<DepartmentResponse>, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<DepartmentResponse>>,
            pub list: Option<Vec<DepartmentResponse>>,
        }

        let resp: Resp = self
            .get(&format!("/api/v1/team/departments/{}/children", id))
            .await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 创建部门
    pub async fn create_department(
        &self,
        req: &CreateDepartmentRequest,
    ) -> Result<i64, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<i64>,
        }

        let resp: Resp = self.post("/api/v1/team/departments", req).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 更新部门
    pub async fn update_department(
        &self,
        id: i64,
        req: &UpdateDepartmentRequest,
    ) -> Result<(), ApiError> {
        self.put(&format!("/api/v1/team/departments/{}", id), req)
            .await
    }

    /// 删除部门
    pub async fn delete_department(&self, id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/api/v1/team/departments/{}", id))
            .await
    }
}

/// 辅助函数：构建查询字符串
fn build_query(query: &ListDepartmentsQuery) -> String {
    let mut parts = Vec::new();

    if let Some(v) = query.org_id {
        parts.push(format!("orgId={}", v));
    }
    if let Some(v) = query.parent_id {
        parts.push(format!("parentId={}", v));
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
        parts.push(format!("pageSize={}", v));
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
