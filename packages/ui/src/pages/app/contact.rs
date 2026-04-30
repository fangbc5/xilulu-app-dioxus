use crate::i18n::use_language;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{
    LdLayers, LdMessageCircle, LdMessageSquare, LdRss, LdSearch, LdStar, LdTag, LdUserPlus, LdUsers,
};
use dioxus_free_icons::Icon;
use rust_i18n::t;

const AVATAR_COLORS: &[&str] = &[
    "#1890ff", "#52c41a", "#fa8c16", "#eb2f96", "#722ed1", "#13c2c2", "#f5222d",
];

#[component]
pub fn AppContact() -> Element {
    let lang = use_language();
    let l_str = lang().as_str();

    let core_app = use_context::<std::sync::Arc<xilulu_im_sdk::service::app::CoreApp>>();
    let mut friends = use_signal(|| core_app.social.contacts_receiver.borrow().clone());

    use_effect({
        let social_service = core_app.social.clone();
        move || {
            let social_clone = social_service.clone();
            spawn(async move {
                if social_clone.contacts_receiver.borrow().is_empty() {
                    let _ = social_clone.load_contacts().await;
                }
            });
        }
    });

    use_effect({
        let rx = core_app.social.contacts_receiver.clone();
        move || {
            let mut my_rx = rx.clone();
            spawn(async move {
                while my_rx.changed().await.is_ok() {
                    let new_contacts = my_rx.borrow().clone();
                    friends.set(new_contacts);
                }
            });
        }
    });

    rsx! {
        div { class: "w-full h-full flex flex-col bg-[#ededed] dark:bg-black pt-[env(safe-area-inset-top)] relative",

            // 微信风格 Header
            header { class: "flex-shrink-0 h-[44px] px-4 flex items-center justify-between transition-colors bg-[#ededed] dark:bg-zinc-900 border-none",
                div { class: "w-8" } // 左侧占位以保持标题居中
                div { class: "flex-1 flex justify-center",
                    span { class: "text-[17px] font-semibold text-zinc-900 dark:text-zinc-100",
                        "{t!(\"nav.contact\", locale = l_str)}"
                    }
                }
                div { class: "w-8 flex items-center justify-end text-zinc-900 dark:text-zinc-100",
                    button { class: "active:opacity-50 transition-opacity outline-none",
                        Icon { width: 22, height: 22, icon: LdUserPlus }
                    }
                }
            }

            // 主要可滚动区域
            main { class: "flex-1 overflow-x-hidden overflow-y-auto overscroll-contain bg-white dark:bg-black relative pb-[calc(50px+env(safe-area-inset-bottom))]",

                // 搜索栏区域 (随内容滑动)
                div { class: "px-2 pb-2 pt-1 bg-[#ededed] dark:bg-zinc-900 border-none",
                    div { class: "w-full h-[36px] bg-white dark:bg-zinc-800 rounded flex items-center justify-center gap-1.5 text-[#b2b2b2] dark:text-zinc-500",
                        Icon { width: 16, height: 16, icon: LdSearch }
                        span { class: "text-[15px]",
                            "{t!(\"contact.searchPlaceholder\", locale = l_str)}"
                        }
                    }
                }

                // 固定功能层 (四大金刚 + 扩展)
                div { class: "bg-white dark:bg-zinc-900",

                    // 新的朋友
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#fa9d3b] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 24,
                                height: 24,
                                icon: LdUserPlus,
                                class: "text-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "{t!(\"contact.newFriend\", locale = l_str)}"
                            }
                        }
                    }
                    // 仅聊天的朋友
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#f09a37] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 22,
                                height: 22,
                                icon: LdMessageSquare,
                                class: "text-white fill-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "仅聊天的朋友"
                            }
                        }
                    }
                    // 群聊
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#07c160] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 24,
                                height: 24,
                                icon: LdUsers,
                                class: "text-white fill-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "{t!(\"contact.groups\", locale = l_str)}"
                            }
                        }
                    }
                    // 标签
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#2782d7] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 24,
                                height: 24,
                                icon: LdTag,
                                class: "text-white fill-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "{t!(\"contact.tags\", locale = l_str)}"
                            }
                        }
                    }
                    // 公众号
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#2782d7] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 22,
                                height: 22,
                                icon: LdRss,
                                class: "text-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "公众号"
                            }
                        }
                    }
                    // 服务号
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#1ebbb8] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 24,
                                height: 24,
                                icon: LdLayers,
                                class: "text-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "服务号"
                            }
                        }
                    }
                    // 企业微信联系人
                    div { class: "w-full flex items-center h-[56px] pl-4 active:bg-zinc-100 dark:active:bg-zinc-800 transition-colors cursor-pointer",
                        div { class: "w-10 h-10 rounded-[4px] bg-[#2782d7] flex items-center justify-center mr-4 flex-shrink-0",
                            Icon {
                                width: 22,
                                height: 22,
                                icon: LdMessageCircle,
                                class: "text-white",
                            }
                        }
                        div { class: "flex-1 h-full flex items-center pr-4",
                            span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100",
                                "企业微信联系人"
                            }
                        }
                    }
                }

                // 好友列表模块
                {
                    let mut current_group = String::new();
                    let friends_list = friends();

                    let nodes: Vec<_> = friends_list.into_iter().map(|friend| {
                        let show_header = if friend.group != current_group {
                            current_group = friend.group.clone();
                            true
                        } else {
                            false
                        };

                        let mut hash: usize = 0;
                        for byte in friend.id.bytes() {
                            hash = hash.wrapping_add(byte as usize);
                        }
                        let bg_color = format!(
                            "background-color: {}",
                            AVATAR_COLORS[hash % AVATAR_COLORS.len()],
                        );
                        
                        rsx! {
                            div { key: "{friend.id}", class: "w-full",
                                if show_header {
                                    div { class: "w-full h-8 px-4 flex items-center bg-[#ededed] dark:bg-[#111111]",
                                        if friend.group == "星标朋友" {
                                            Icon {
                                                width: 14,
                                                height: 14,
                                                icon: LdStar,
                                                class: "text-zinc-400 mr-2",
                                            }
                                        }
                                        span { class: "text-[13px] text-[#767676] dark:text-zinc-500", "{friend.group}" }
                                    }
                                }
        
                                div { class: "w-full flex items-center h-[56px] pl-4 bg-white active:bg-zinc-100 dark:bg-zinc-900 dark:active:bg-zinc-800 transition-colors cursor-pointer",
        
                                    div { class: "w-10 h-10 rounded-[4px] overflow-hidden mr-4 flex-shrink-0 flex items-center justify-center text-white text-base font-semibold",
                                        if let Some(av) = &friend.avatar {
                                            img {
                                                src: "{av}",
                                                class: "w-full h-full object-cover rounded-[4px]",
                                            }
                                        } else {
                                            div {
                                                class: "w-full h-full flex items-center justify-center rounded-[4px]",
                                                style: "{bg_color}",
                                                "{friend.name.chars().next().unwrap_or('?')}"
                                            }
                                        }
                                    }
        
                                    div { class: "flex-1 h-full flex items-center pr-4 border-b border-[#f3f3f3] dark:border-zinc-800/50",
                                        span { class: "text-[16.5px] text-zinc-900 dark:text-zinc-100", "{friend.name}" }
                                    }
                                }
                            }
                        }
                    }).collect();

                    rsx! {
                        {nodes.into_iter()}
                    }
                }

                // 底部计数器
                div { class: "w-full flex items-center justify-center py-6 pb-20 bg-white dark:bg-zinc-900",
                    span { class: "text-[15px] text-[#767676] dark:text-zinc-500", "225个朋友" }
                }
            }

            // 侧边悬浮字母索引表
            div { class: "absolute right-0.5 top-[50%] -translate-y-1/2 flex flex-col items-center justify-center z-10 w-6",
                div { class: "text-[10px] sm:text-[11px] text-[#555555] dark:text-zinc-500 leading-tight py-0.5 mt-2",
                    Icon { width: 10, height: 10, icon: LdSearch }
                }
                div { class: "text-[10px] sm:text-[11px] text-[#555555] dark:text-zinc-500 leading-tight py-0.5",
                    Icon { width: 10, height: 10, icon: LdStar }
                }
                for c in vec![
                    "A",
                    "B",
                    "C",
                    "D",
                    "E",
                    "F",
                    "G",
                    "H",
                    "I",
                    "J",
                    "K",
                    "L",
                    "M",
                    "N",
                    "O",
                    "P",
                    "Q",
                    "R",
                    "S",
                    "T",
                    "U",
                    "V",
                    "W",
                    "X",
                    "Y",
                    "Z",
                    "#",
                ]
                {
                    div { class: "text-[9px] sm:text-[10px] font-medium leading-[1.2] py-[1px] text-[#555555] dark:text-zinc-500",
                        if c == "Z" {
                            // 为了截图中Z绿色的高亮效果
                            div { class: "w-3.5 h-3.5 rounded-full bg-[#07c160] text-white flex items-center justify-center font-bold",
                                "{c}"
                            }
                        } else {
                            "{c}"
                        }
                    }
                }
            }
        }
    }
}
