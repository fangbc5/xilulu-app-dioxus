//! 认证页面
//!
//! 登录页面（含图片验证码）和租户选择页面。

use dioxus::prelude::*;

use crate::services::auth::*;
use crate::services::client::ApiClient;
use crate::services::storage::save_auth;

/// 登录页面
#[component]
pub fn LoginPage() -> Element {
    let nav = use_navigator();
    let api_client = use_context::<ApiClient>();
    let mut stage = use_context::<Signal<AuthStage>>();
    let mut access_token = use_context::<Signal<String>>();
    let mut refresh_token = use_context::<Signal<String>>();
    let mut user_info: Signal<Option<UserInfo>> = use_context();

    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut captcha_input = use_signal(String::new);
    let mut captcha_id = use_signal(String::new);
    let mut captcha_img = use_signal(String::new);
    let mut error_msg = use_signal(|| String::new());
    let mut loading = use_signal(|| false);

    // 首次加载验证码
    let api_clone = api_client.clone();
    use_effect(move || {
        let api = api_clone.clone();
        spawn(async move {
            match api.auth_get_captcha().await {
                Ok(resp) => {
                    captcha_id.set(resp.captcha_id);
                    captcha_img.set(resp.image_base64);
                }
                Err(e) => {
                    error_msg.set(format!("获取验证码失败：{e}"));
                }
            }
        });
    });

    let opacity = if loading() { "0.7" } else { "1" };

    rsx! {
        div { style: "
                display: flex; align-items: center; justify-content: center;
                min-height: 100vh; width: 100%;
                background: var(--ds-bg-primary);
            ",

            div {
                class: "glass-card",
                style: "
                    width: 300px; padding: 24px 20px;
                    display: flex; flex-direction: column; gap: 14px;
                ",

                // Logo
                div { style: "text-align: center;",
                    h1 {
                        class: "aurora-text",
                        style: "font-size: 20px; font-weight: 600; letter-spacing: -0.02em;",
                        "✦ Xilulu"
                    }
                }

                // 表单
                div { style: "display: flex; flex-direction: column; gap: 10px;",

                    // 用户名
                    input {
                        r#type: "text",
                        value: "{username}",
                        placeholder: "用户名",
                        style: "
                            width: 100%; padding: 9px 12px;
                            background: var(--ds-bg-secondary);
                            border: 1px solid rgba(148, 163, 184, 0.4);
                            border-radius: 6px; font-size: 13px;
                            color: var(--ds-text-primary);
                            outline: none; box-sizing: border-box;
                        ",
                        oninput: move |e| username.set(e.value()),
                    }

                    // 密码
                    input {
                        r#type: "password",
                        value: "{password}",
                        placeholder: "密码",
                        style: "
                            width: 100%; padding: 9px 12px;
                            background: var(--ds-bg-secondary);
                            border: 1px solid rgba(148, 163, 184, 0.4);
                            border-radius: 6px; font-size: 13px;
                            color: var(--ds-text-primary);
                            outline: none; box-sizing: border-box;
                        ",
                        oninput: move |e| password.set(e.value()),
                    }

                    // 验证码行（输入框全宽 + 图片绝对定位在右侧）
                    div { style: "position: relative;",
                        input {
                            r#type: "text",
                            value: "{captcha_input}",
                            placeholder: "验证码",
                            style: "
                                width: 100%; padding: 9px 110px 9px 12px;
                                background: var(--ds-bg-secondary);
                                border: 1px solid rgba(148, 163, 184, 0.4);
                                border-radius: 6px; font-size: 13px;
                                color: var(--ds-text-primary);
                                outline: none; box-sizing: border-box;
                            ",
                            oninput: move |e| captcha_input.set(e.value()),
                        }

                        // 验证码图片，绝对定位在输入框右侧
                        if !captcha_img().is_empty() {
                            img {
                                src: "data:image/png;base64,{captcha_img}",
                                style: "
                                    position: absolute; right: 2px; top: 50%;
                                    transform: translateY(-50%);
                                    height: 30px; width: 96px;
                                    border-radius: 4px; cursor: pointer;
                                ",
                                onclick: {
                                    let api = api_client.clone();
                                    move |_| {
                                        let api = api.clone();
                                        spawn(async move {
                                            match api.auth_get_captcha().await {
                                                Ok(resp) => {
                                                    captcha_id.set(resp.captcha_id);
                                                    captcha_img.set(resp.image_base64);
                                                }
                                                Err(e) => {
                                                    error_msg.set(format!("获取验证码失败：{e}"));
                                                }
                                            }
                                        });
                                    }
                                },
                                title: "点击刷新验证码",
                            }
                        }
                    }

                    // 错误提示
                    if !error_msg().is_empty() {
                        div { style: "
                                padding: 8px 12px;
                                background: rgba(239, 68, 68, 0.1);
                                border: 1px solid rgba(239, 68, 68, 0.2);
                                border-radius: 6px;
                                font-size: 12px; color: #ef4444;
                            ",
                            "{error_msg}"
                        }
                    }

                    // 登录按钮
                    button {
                        disabled: loading(),
                        style: "
                            width: 100%; padding: 10px;
                            background: var(--ds-accent); color: white;
                            border: none; border-radius: 6px;
                            font-size: 13px; font-weight: 500;
                            cursor: pointer; opacity: {opacity};
                            transition: opacity 150ms ease;
                        ",
                        onclick: {
                            let api = api_client.clone();
                            move |_| {
                                let user = username();
                                let pass = password();
                                let cap = captcha_input();
                                let cap_id = captcha_id();
                                if user.is_empty() || pass.is_empty() {
                                    error_msg.set("请输入用户名和密码".to_string());
                                    return;
                                }
                                if cap.is_empty() || cap_id.is_empty() {
                                    error_msg.set("请输入验证码".to_string());
                                    return;
                                }
                                loading.set(true);
                                error_msg.set(String::new());

                                let api = api.clone();
                                let nav = nav.clone();
                                spawn(async move {
                                    let req = LoginRequest {
                                        username: Some(user),
                                        password: Some(pass),
                                        mobile: None,
                                        email: None,
                                        code: None,
                                        captcha_id: Some(cap_id),
                                        captcha: Some(cap),
                                        region: None,
                                    };
                                    match api.auth_login(&req).await {
                                        Ok(resp) => {
                                            if let Some(tenants) = &resp.tenant_list {
                                                if tenants.len() > 1 {
                                                    refresh_token.set(resp.refresh_token);
                                                    user_info.set(Some(resp.user_info));
                                                    let new_stage = AuthStage::NeedTenant {
                                                        temp_token: resp.access_token,
                                                        tenant_list: tenants.clone(),
                                                    };
                                                    stage.set(new_stage);
                                                    nav.push("/select-tenant");
                                                    return;
                                                }
                                            }
                                            api.set_token(&resp.access_token);
                                            api.set_refresh_token(&resp.refresh_token);
                                            access_token.set(resp.access_token.clone());
                                            refresh_token.set(resp.refresh_token.clone());
                                            let ui = resp.user_info.clone();
                                            user_info.set(Some(resp.user_info));
                                            stage.set(AuthStage::Authenticated);
                                            save_auth(&access_token(), &refresh_token(), &ui);
                                            nav.push("/");
                                        }
                                        Err(e) => {
                                            error_msg.set(format!("登录失败：{e}"));
                                            // 刷新验证码
                                            match api.auth_get_captcha().await {
                                                Ok(r) => {
                                                    captcha_id.set(r.captcha_id);
                                                    captcha_img.set(r.image_base64);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    loading.set(false);
                                });
                            }
                        },
                        if loading() {
                            "登录中..."
                        } else {
                            "登录"
                        }
                    }
                }

                // 底部
                p { style: "text-align: center; font-size: 12px; color: var(--ds-text-tertiary);",
                    "Xilulu Console v0.1"
                }
            }
        }
    }
}

/// 选租户页面
#[component]
pub fn SelectTenantPage() -> Element {
    let nav = use_navigator();
    let api_client = use_context::<ApiClient>();
    let mut stage = use_context::<Signal<AuthStage>>();
    let mut access_token = use_context::<Signal<String>>();
    let mut refresh_token = use_context::<Signal<String>>();
    let mut user_info: Signal<Option<UserInfo>> = use_context();

    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| String::new());

    let tenant_data = match stage() {
        AuthStage::NeedTenant { temp_token, tenant_list } => Some((temp_token, tenant_list)),
        _ => None,
    };

    if tenant_data.is_none() {
        return rsx! {
            div { style: "display: flex; align-items: center; justify-content: center; min-height: 100vh;",
                p { style: "color: var(--ds-text-secondary);", "正在跳转..." }
            }
        };
    }

    let (temp_token, tenant_list) = tenant_data.unwrap();

    rsx! {
        div { style: "
                display: flex; align-items: center; justify-content: center;
                min-height: 100vh; width: 100%;
                background: var(--ds-bg-primary);
            ",

            div {
                class: "glass-card",
                style: "
                    width: 360px; padding: 28px 24px;
                    display: flex; flex-direction: column; gap: 16px;
                ",

                div { style: "text-align: center;",
                    h1 {
                        class: "aurora-text",
                        style: "font-size: 18px; font-weight: 600;",
                        "选择组织"
                    }
                    p { style: "font-size: 12px; color: var(--ds-text-tertiary); margin-top: 4px;",
                        "你属于多个组织，请选择要进入的组织"
                    }
                }

                if !error_msg().is_empty() {
                    div { style: "
                            padding: 10px 14px;
                            background: rgba(239, 68, 68, 0.1);
                            border: 1px solid rgba(239, 68, 68, 0.2);
                            border-radius: 8px;
                            font-size: 13px; color: #ef4444;
                        ",
                        "{error_msg}"
                    }
                }

                div { style: "display: flex; flex-direction: column; gap: 10px;",
                    for tenant in tenant_list.iter() {
                        {
                            let tenant_id_str = tenant.id.clone();
                            let tenant_name = tenant.name.clone();
                            let tenant_id: i64 = tenant_id_str.parse().unwrap_or(0);
                            let is_owner = tenant.is_owner.unwrap_or(false);
                            let token = temp_token.clone();
                            rsx! {
                                button {
                                    disabled: loading(),
                                    style: "
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        display: flex; align-items: center; justify-content: space-between;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        width: 100%; padding: 10px 14px;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        background: var(--ds-bg-secondary);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        border: 1px solid var(--ds-border-primary);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        border-radius: 8px; cursor: pointer;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        transition: all 150ms ease;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        text-align: left;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                    ",
                                    onclick: {
                                        let api = api_client.clone();
                                        move |_| {
                                            loading.set(true);
                                            error_msg.set(String::new());
                                            let api = api.clone();
                                            let nav = nav.clone();
                                            let token = token.clone();
                                            spawn(async move {
                                                let req = SelectTenantRequest {
                                                    tenant_id,
                                                    temp_token: token,
                                                };
                                                match api.auth_select_tenant(&req).await {
                                                    Ok(resp) => {
                                                        api.set_token(&resp.access_token);
                                                        api.set_refresh_token(&resp.refresh_token);
                                                        access_token.set(resp.access_token.clone());
                                                        refresh_token.set(resp.refresh_token.clone());
                                                        let ui = resp.user_info.clone();
                                                        user_info.set(Some(resp.user_info));
                                                        stage.set(AuthStage::Authenticated);
                                                        save_auth(&access_token(), &refresh_token(), &ui);
                                                        nav.push("/");
                                                    }
                                                    Err(e) => {
                                                        error_msg.set(format!("选择组织失败：{e}"));
                                                    }
                                                }
                                                loading.set(false);
                                            });
                                        }
                                    },

                                    div {
                                        div { style: "font-size: 13px; font-weight: 500; color: var(--ds-text-primary);",
                                            "{tenant_name}"
                                        }
                                    }

                                    if is_owner {
                                        span {
                                            style: "
                                                                                                                                                                                                                                                                                                                                                                                                                                                                            font-size: 11px; padding: 2px 8px;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                            background: rgba(59, 130, 246, 0.1);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                            color: #3b82f6; border-radius: 4px;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                        ",
                                            "所有者"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "text-align: center;",
                    button {
                        style: "
                            font-size: 13px; color: var(--ds-text-tertiary);
                            background: none; border: none; cursor: pointer;
                        ",
                        onclick: move |_| {
                            stage.set(AuthStage::Unauthenticated);
                            nav.push("/login");
                        },
                        "← 返回登录"
                    }
                }
            }
        }
    }
}