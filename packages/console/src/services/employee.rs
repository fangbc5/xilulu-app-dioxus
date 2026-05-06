//! 员工管理 API
//!
//! 对接 ms-team /api/v1/team/employees

use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

/// ============ DTO 定义 ============

/// 创建员工请求
#[derive(Debug, Serialize)]
pub struct CreateEmployeeRequest {
    #[serde(rename = "orgId")]
    pub org_id: i64,
    #[serde(rename = "userId")]
    pub user_id: Option<i64>,
    #[serde(rename = "employeeNo")]
    pub employee_no: Option<String>,
    pub name: String,
    pub avatar: Option<String>,
    pub gender: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "hireDate")]
    pub hire_date: Option<String>,
    #[serde(rename = "primaryDepartmentId")]
    pub primary_department_id: Option<i64>,
    #[serde(rename = "primaryPositionId")]
    pub primary_position_id: Option<i64>,
}

/// 更新员工请求
#[derive(Debug, Serialize)]
pub struct UpdateEmployeeRequest {
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub gender: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "hireDate")]
    pub hire_date: Option<String>,
    #[serde(rename = "leaveDate")]
    pub leave_date: Option<String>,
    pub status: Option<i16>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
}

/// 添加员工到部门请求
#[derive(Debug, Serialize)]
pub struct AddEmployeeToDepartmentRequest {
    #[serde(rename = "departmentId")]
    pub department_id: i64,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
    #[serde(rename = "isLeader")]
    pub is_leader: Option<bool>,
}

/// 添加员工岗位请求
#[derive(Debug, Serialize)]
pub struct AddEmployeePositionRequest {
    #[serde(rename = "positionId")]
    pub position_id: i64,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
}

/// 员工响应
#[derive(Debug, Deserialize)]
pub struct EmployeeResponse {
    pub id: i64,
    #[serde(rename = "orgId")]
    pub org_id: i64,
    #[serde(rename = "userId")]
    pub user_id: Option<i64>,
    #[serde(rename = "employeeNo")]
    pub employee_no: Option<String>,
    pub name: String,
    pub avatar: Option<String>,
    pub gender: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub status: Option<i16>,
    #[serde(rename = "primaryDepartmentId")]
    pub primary_department_id: Option<i64>,
    #[serde(rename = "primaryDepartmentName")]
    pub primary_department_name: Option<String>,
    #[serde(rename = "primaryPositionId")]
    pub primary_position_id: Option<i64>,
    #[serde(rename = "primaryPositionName")]
    pub primary_position_name: Option<String>,
    #[serde(rename = "hireDate")]
    pub hire_date: Option<String>,
    #[serde(rename = "leaveDate")]
    pub leave_date: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i32>,
    #[serde(rename = "createTime")]
    pub create_time: Option<String>,
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
}

/// 员工列表查询
#[derive(Debug, Serialize)]
pub struct ListEmployeesQuery {
    #[serde(rename = "orgId")]
    pub org_id: Option<i64>,
    #[serde(rename = "departmentId")]
    pub department_id: Option<i64>,
    #[serde(rename = "includeChildren")]
    pub include_children: Option<bool>,
    #[serde(rename = "positionId")]
    pub position_id: Option<i64>,
    pub status: Option<i16>,
    pub keyword: Option<String>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u32>,
    pub cursor: Option<u32>,
}

/// ============ API 方法 ============

impl ApiClient {
    /// 获取员工列表
    pub async fn list_employees(
        &self,
        query: &ListEmployeesQuery,
    ) -> Result<Vec<EmployeeResponse>, ApiError> {
        let query_str = build_employee_query(query);
        let path = format!("/api/v1/team/employees{}", query_str);

        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<Vec<EmployeeResponse>>,
            pub list: Option<Vec<EmployeeResponse>>,
        }

        let resp: Resp = self.get(&path).await?;
        Ok(resp.data.or(resp.list).unwrap_or_default())
    }

    /// 获取单个员工详情
    pub async fn get_employee(&self, id: i64) -> Result<EmployeeResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<EmployeeResponse>,
        }

        let resp: Resp = self.get(&format!("/api/v1/team/employees/{}", id)).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 404,
            message: "Employee not found".to_string(),
        })
    }

    /// 创建员工
    pub async fn create_employee(
        &self,
        req: &CreateEmployeeRequest,
    ) -> Result<i64, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<i64>,
        }

        let resp: Resp = self.post("/api/v1/team/employees", req).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 更新员工
    pub async fn update_employee(
        &self,
        id: i64,
        req: &UpdateEmployeeRequest,
    ) -> Result<(), ApiError> {
        self.put(&format!("/api/v1/team/employees/{}", id), req)
            .await
    }

    /// 删除员工
    pub async fn delete_employee(&self, id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/api/v1/team/employees/{}", id))
            .await
    }

    /// 添加员工到部门
    pub async fn add_employee_to_department(
        &self,
        employee_id: i64,
        req: &AddEmployeeToDepartmentRequest,
    ) -> Result<i64, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<i64>,
        }

        let resp: Resp = self
            .post(
                &format!("/api/v1/team/employees/{}/departments", employee_id),
                req,
            )
            .await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 从部门移除员工
    pub async fn remove_employee_from_department(
        &self,
        employee_id: i64,
        department_id: i64,
    ) -> Result<(), ApiError> {
        self.delete(&format!(
            "/api/v1/team/employees/{}/departments/{}",
            employee_id, department_id
        ))
        .await
    }

    /// 添加员工岗位
    pub async fn add_employee_position(
        &self,
        employee_id: i64,
        req: &AddEmployeePositionRequest,
    ) -> Result<i64, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<i64>,
        }

        let resp: Resp = self
            .post(
                &format!("/api/v1/team/employees/{}/positions", employee_id),
                req,
            )
            .await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 移除员工岗位
    pub async fn remove_employee_position(
        &self,
        employee_id: i64,
        position_id: i64,
    ) -> Result<(), ApiError> {
        self.delete(&format!(
            "/api/v1/team/employees/{}/positions/{}",
            employee_id, position_id
        ))
        .await
    }
}

/// 辅助函数：构建查询字符串
fn build_employee_query(query: &ListEmployeesQuery) -> String {
    let mut parts = Vec::new();

    if let Some(v) = query.org_id {
        parts.push(format!("orgId={}", v));
    }
    if let Some(v) = query.department_id {
        parts.push(format!("departmentId={}", v));
    }
    if let Some(v) = query.include_children {
        parts.push(format!("includeChildren={}", v));
    }
    if let Some(v) = query.position_id {
        parts.push(format!("positionId={}", v));
    }
    if let Some(v) = query.status {
        parts.push(format!("status={}", v));
    }
    if let Some(ref v) = query.keyword {
        if !v.is_empty() {
            parts.push(format!("keyword={}", url_encode(v)));
        }
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
