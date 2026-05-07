//! 意图执行器
//!
//! 将解析后的意图转化为实际的 API 调用。
//! 每个执行成功后会返回执行摘要。

use crate::intent::ParsedIntent;
use crate::services::client::ApiClient;
use crate::services::department::{CreateDepartmentRequest, ListDepartmentsQuery};
use crate::services::employee::{AddEmployeeToDepartmentRequest, ListEmployeesQuery};

/// 执行结果
#[derive(Clone, Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub summary: String,
    pub detail: Option<String>,
}

/// 执行一个已确认的意图
pub async fn execute_intent(
    client: &ApiClient,
    intent: &ParsedIntent,
    org_id: i64,
) -> ExecutionResult {
    match intent {
        ParsedIntent::CreateDepartment {
            department_name,
            parent_department,
        } => execute_create_department(client, department_name.clone(), parent_department.clone(), org_id).await,

        ParsedIntent::Transfer {
            employee_name,
            from_department,
            to_department,
        } => {
            execute_transfer(
                client,
                employee_name.clone(),
                from_department.clone(),
                to_department.clone(),
                org_id,
            ).await
        }

        ParsedIntent::Search { keyword, scope } => {
            let scope_text = match scope {
                crate::intent::SearchScope::Employee => "员工",
                crate::intent::SearchScope::Department => "部门",
                crate::intent::SearchScope::All => "",
            };
            // 搜索不需要真正执行，只是导航
            ExecutionResult {
                success: true,
                summary: format!("搜索{scope_text}：{keyword}"),
                detail: None,
            }
        }

        ParsedIntent::Cancel => ExecutionResult {
            success: true,
            summary: "已取消".to_string(),
            detail: None,
        },

        ParsedIntent::Unknown { message } => ExecutionResult {
            success: false,
            summary: format!("无法执行：{message}"),
            detail: None,
        },
    }
}

/// 执行创建部门
async fn execute_create_department(
    client: &ApiClient,
    department_name: Option<String>,
    parent_department_name: Option<String>,
    org_id: i64,
) -> ExecutionResult {
    let name = match department_name {
        Some(n) if !n.is_empty() => n,
        _ => {
            return ExecutionResult {
                success: false,
                summary: "缺少部门名称".to_string(),
                detail: None,
            };
        }
    };

    // 1. 查找上级部门 ID（如果指定了上级部门）
    let parent_id = if let Some(ref parent_name) = parent_department_name {
        match find_department_by_name(client, parent_name, org_id).await {
            Ok(Some(dept)) => Some(dept.id),
            Ok(None) => {
                return ExecutionResult {
                    success: false,
                    summary: format!("找不到上级部门「{parent_name}」"),
                    detail: Some("请确认部门名称是否正确".to_string()),
                };
            }
            Err(e) => {
                return ExecutionResult {
                    success: false,
                    summary: format!("查询上级部门失败: {e}"),
                    detail: None,
                };
            }
        }
    } else {
        None
    };

    // 2. 生成部门编码（用名称的拼音首字母或简单规则）
    let code = generate_dept_code(&name);

    // 3. 调用创建 API
    let req = CreateDepartmentRequest {
        org_id,
        parent_id,
        code: code.clone(),
        name: name.clone(),
        leader_employee_id: None,
        sort_order: None,
    };

    match client.create_department(&req).await {
        Ok(id) => {
            let parent_info = parent_department_name
                .as_ref()
                .map(|p| format!("（上级: {p}）"))
                .unwrap_or_default();
            ExecutionResult {
                success: true,
                summary: format!("✅ 部门「{name}」创建成功{parent_info}"),
                detail: Some(format!("部门ID: {id}, 编码: {code}")),
            }
        }
        Err(e) => ExecutionResult {
            success: false,
            summary: format!("❌ 创建部门「{name}」失败"),
            detail: Some(format!("错误: {e}")),
        },
    }
}

/// 根据名称查找部门
async fn find_department_by_name(
    client: &ApiClient,
    name: &str,
    org_id: i64,
) -> Result<Option<crate::services::department::DepartmentResponse>, String> {
    let query = ListDepartmentsQuery {
        org_id: Some(org_id),
        parent_id: None,
        keyword: Some(name.to_string()),
        status: None,
        page_size: Some(10),
        cursor: None,
    };

    client
        .list_departments(&query)
        .await
        .map_err(|e| e.to_string())
        .map(|list| {
            // 精确匹配优先
            list.into_iter().find(|d| d.name == name)
        })
}

/// 执行调岗操作
async fn execute_transfer(
    client: &ApiClient,
    employee_name: Option<String>,
    from_department: Option<String>,
    to_department: Option<String>,
    org_id: i64,
) -> ExecutionResult {
    let emp_name = match employee_name {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => {
            return ExecutionResult {
                success: false,
                summary: "缺少员工姓名".to_string(),
                detail: None,
            };
        }
    };
    let to_dept_name = match to_department {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => {
            return ExecutionResult {
                success: false,
                summary: "缺少目标部门名称".to_string(),
                detail: None,
            };
        }
    };

    // 1. 查找员工
    let emp_query = ListEmployeesQuery {
        org_id: Some(org_id),
        department_id: None,
        include_children: None,
        position_id: None,
        status: None,
        keyword: Some(emp_name.clone()),
        page_size: Some(10),
        cursor: None,
    };
    let employees = match client.list_employees(&emp_query).await {
        Ok(list) => list,
        Err(e) => {
            return ExecutionResult {
                success: false,
                summary: format!("查询员工失败: {e}"),
                detail: None,
            };
        }
    };

    let employee = match employees.into_iter().find(|e| e.name == emp_name) {
        Some(e) => e,
        None => {
            return ExecutionResult {
                success: false,
                summary: format!("找不到员工「{emp_name}」"),
                detail: Some("请确认员工姓名是否正确".to_string()),
            };
        }
    };

    // 2. 查找目标部门
    let to_dept = match find_department_by_name(client, &to_dept_name, org_id).await {
        Ok(Some(d)) => d,
        Ok(None) => {
            return ExecutionResult {
                success: false,
                summary: format!("找不到目标部门「{to_dept_name}」"),
                detail: Some("请确认部门名称是否正确".to_string()),
            };
        }
        Err(e) => {
            return ExecutionResult {
                success: false,
                summary: format!("查询目标部门失败: {e}"),
                detail: None,
            };
        }
    };

    // 3. 如果有原部门，先从原部门移除
    if let Some(ref from_dept_name) = from_department {
        if let Ok(Some(from_dept)) = find_department_by_name(client, from_dept_name, org_id).await {
            let _ = client.remove_employee_from_department(employee.id, from_dept.id).await;
        }
    } else if let Some(primary_dept_id) = employee.primary_department_id {
        // 没指定原部门时，从当前主部门移除
        let _ = client.remove_employee_from_department(employee.id, primary_dept_id).await;
    }

    // 4. 添加到目标部门
    let req = AddEmployeeToDepartmentRequest {
        department_id: to_dept.id,
        is_primary: Some(true),
        is_leader: None,
    };

    match client.add_employee_to_department(employee.id, &req).await {
        Ok(_) => {
            let from_text = from_department
                .as_deref()
                .or(employee.primary_department_name.as_deref())
                .unwrap_or("原部门");
            ExecutionResult {
                success: true,
                summary: format!("✅ 已将 {emp_name} 从「{from_text}」调到「{to_dept_name}」"),
                detail: Some(format!(
                    "员工ID: {}, 目标部门ID: {}",
                    employee.id, to_dept.id
                )),
            }
        }
        Err(e) => ExecutionResult {
            success: false,
            summary: format!("❌ 调岗失败：{emp_name} → {to_dept_name}"),
            detail: Some(format!("错误: {e}")),
        },
    }
}

/// 简单的部门编码生成
fn generate_dept_code(name: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("DEPT-{ts:x}")
}
