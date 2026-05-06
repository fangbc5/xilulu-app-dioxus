//! 通讯录 API
//!
//! 对接 ms-team /api/v1/team/contacts

use serde::Deserialize;

use super::client::{ApiClient, ApiError};

/// ============ DTO 定义 ============

/// 通讯录入口响应
#[derive(Debug, Deserialize)]
pub struct ContactsEntryResponse {
    #[serde(rename = "organization")]
    pub organization: OrganizationBrief,
    pub departments: Vec<DepartmentSummary>,
    #[serde(rename = "totalMemberCount")]
    pub total_member_count: Option<i64>,
}

/// 组织简要信息
#[derive(Debug, Deserialize)]
pub struct OrganizationBrief {
    pub id: i64,
    pub name: String,
    pub logo: Option<String>,
}

/// 部门摘要
#[derive(Debug, Deserialize)]
pub struct DepartmentSummary {
    pub id: i64,
    pub name: String,
    #[serde(rename = "hasChildren")]
    pub has_children: Option<bool>,
    #[serde(rename = "memberCount")]
    pub member_count: Option<i64>,
    pub leader: Option<LeaderBrief>,
}

/// 负责人简要信息
#[derive(Debug, Deserialize)]
pub struct LeaderBrief {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
}

/// 部门详情响应
#[derive(Debug, Deserialize)]
pub struct ContactsDepartmentResponse {
    pub department: DepartmentInfo,
    pub children: Option<Vec<DepartmentSummary>>,
    pub members: Option<Vec<MemberPreview>>,
    #[serde(rename = "totalMemberCount")]
    pub total_member_count: Option<i64>,
    #[serde(rename = "subDeptCount")]
    pub sub_dept_count: Option<i64>,
}

/// 部门信息
#[derive(Debug, Deserialize)]
pub struct DepartmentInfo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "leaderName")]
    pub leader_name: Option<String>,
}

/// 成员预览
#[derive(Debug, Deserialize)]
pub struct MemberPreview {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    #[serde(rename = "departmentTitle")]
    pub department_title: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "isLeader")]
    pub is_leader: Option<bool>,
}

/// 员工通讯录详情
#[derive(Debug, Deserialize)]
pub struct ContactsEmployeeDetailResponse {
    pub id: i64,
    pub name: String,
    pub avatar: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    pub departments: Vec<EmployeeDeptInfo>,
    pub positions: Vec<EmployeePosInfo>,
}

/// 员工部门信息
#[derive(Debug, Deserialize)]
pub struct EmployeeDeptInfo {
    pub id: i64,
    pub name: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
    #[serde(rename = "isLeader")]
    pub is_leader: Option<bool>,
}

/// 员工岗位信息
#[derive(Debug, Deserialize)]
pub struct EmployeePosInfo {
    pub id: i64,
    pub name: String,
    pub level: Option<i32>,
    #[serde(rename = "isPrimary")]
    pub is_primary: Option<bool>,
}

/// 通讯录搜索响应
#[derive(Debug, Deserialize)]
pub struct ContactsSearchResponse {
    pub items: Vec<SearchResultItem>,
    #[serde(rename = "estimatedTotal")]
    pub estimated_total: Option<i64>,
    #[serde(rename = "hasNext")]
    pub has_next: Option<bool>,
    pub degraded: Option<bool>,
}

/// 搜索结果项
#[derive(Debug, Deserialize)]
pub struct SearchResultItem {
    #[serde(rename = "employeeId")]
    pub employee_id: i64,
    pub name: String,
    pub avatar: Option<String>,
    #[serde(rename = "primaryDeptName")]
    pub primary_dept_name: Option<String>,
    #[serde(rename = "primaryPositionName")]
    pub primary_position_name: Option<String>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "highlights")]
    pub highlights: Option<Vec<String>>,
}

/// ============ API 方法 ============

impl ApiClient {
    /// 获取通讯录入口（组织 + 根部门）
    pub async fn contacts_entry(&self, org_id: i64) -> Result<ContactsEntryResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<ContactsEntryResponse>,
        }

        let resp: Resp = self
            .get(&format!("/api/v1/team/contacts/entry?orgId={}", org_id))
            .await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 获取部门通讯录（子部门 + 成员预览）
    pub async fn contacts_department(
        &self,
        dept_id: i64,
    ) -> Result<ContactsDepartmentResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<ContactsDepartmentResponse>,
        }

        let resp: Resp = self
            .get(&format!(
                "/api/v1/team/contacts/departments/{}",
                dept_id
            ))
            .await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 获取员工通讯录详情
    pub async fn contacts_employee_detail(
        &self,
        id: i64,
    ) -> Result<ContactsEmployeeDetailResponse, ApiError> {
        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<ContactsEmployeeDetailResponse>,
        }

        let resp: Resp = self
            .get(&format!("/api/v1/team/contacts/employees/{}", id))
            .await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 搜索通讯录
    pub async fn contacts_search(
        &self,
        org_id: i64,
        keyword: &str,
    ) -> Result<ContactsSearchResponse, ApiError> {
        let encoded_keyword = url_encode(keyword);
        let path = format!(
            "/api/v1/team/contacts/search?orgId={}&keyword={}",
            org_id, encoded_keyword
        );

        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<ContactsSearchResponse>,
        }

        let resp: Resp = self.get(&path).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }

    /// 获取部门成员（分页）
    pub async fn contacts_department_members(
        &self,
        dept_id: i64,
        include_children: Option<bool>,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<ContactsMemberPageResponse, ApiError> {
        let mut parts = Vec::new();
        parts.push(format!("deptId={}", dept_id));
        if let Some(v) = include_children {
            parts.push(format!("includeChildren={}", v));
        }
        if let Some(v) = page {
            parts.push(format!("page={}", v));
        }
        if let Some(v) = page_size {
            parts.push(format!("pageSize={}", v));
        }

        let path = format!("/api/v1/team/contacts/departments/{}/members?{}", dept_id, parts.join("&"));

        #[derive(Debug, Deserialize)]
        struct Resp {
            pub data: Option<ContactsMemberPageResponse>,
        }

        let resp: Resp = self.get(&path).await?;
        resp.data.ok_or_else(|| ApiError::Server {
            status: 500,
            message: "No data returned".to_string(),
        })
    }
}

/// 部门成员分页响应
#[derive(Debug, Deserialize)]
pub struct ContactsMemberPageResponse {
    pub items: Vec<MemberPreview>,
    pub total: Option<i64>,
    #[serde(rename = "hasNext")]
    pub has_next: Option<bool>,
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
