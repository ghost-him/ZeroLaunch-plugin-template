//! Hello World 示例插件——**业务文件**：不参与模板同步（`.templatesyncignore` 排除 `src/plugin.rs`）。
//!
//! 照常实现宿主的 `Plugin` / `Configurable` trait；启动骨架（`src/main.rs`）只负责 `init()` + `app().run()`。
//! 面向用户的文本用 `t_key("key")` 生成命名空间键（`plugin.<插件id>.<key>`），译文放 `i18n/<lang>.json`。

use std::sync::Arc;

use async_trait::async_trait;
use zerolaunch_plugin_api::config::{
    ComponentCore, ComponentType, Configurable, SettingDefinition,
};
use zerolaunch_plugin_api::{
    Plugin, PluginContext, PluginError, PluginHandle, PluginKind, PluginMetadata,
    Query, QueryResponse, ListItem, ResultAction,
};
use zerolaunch_plugin_protocol::Manifest;
use zerolaunch_plugin_sdk_rust::{PluginApp, t_key};

/// 读取插件目录下的 `manifest.toml`。
///
/// 宿主 spawn 插件时把工作目录设为插件目录，故清单与插件进程同源；解析用
/// `zerolaunch_plugin_protocol::Manifest`（宿主同一套 schema），
/// 必填字段缺一即解析失败并退出（宿主侧表现为插件加载失败）。
fn read_manifest() -> Manifest {
    let path = "manifest.toml";
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("读取插件清单 {path} 失败：{e}（请在插件目录下运行插件）"));
    toml::from_str(&raw).unwrap_or_else(|e| panic!("解析插件清单 {path} 失败：{e}"))
}

/// Hello World 示例插件 — 演示第三方插件的最小骨架与 i18n 用法。
///
/// 仅作为插件模板使用；文本经 `t_key()` 生成命名空间翻译键
/// （`plugin.com.example.hello-world.<key>`），宿主加载插件目录
/// `i18n/<lang>.json` 语言包后，前端对 key-or-literal 文本自动翻译。
struct HelloWorldPlugin {
    /// 组件 ID、名称、类型等基础元数据（`Configurable` trait 默认实现委托于此）。
    core: ComponentCore,
    /// 插件静态元数据：id、触发关键词、优先级等。
    metadata: PluginMetadata,
}

impl HelloWorldPlugin {
    fn new() -> Self {
        // 元数据全部来自清单：必填字段由清单 schema 强制，解析通过即齐备
        let plugin = read_manifest().plugin;
        Self {
            core: ComponentCore::new(
                plugin.id.clone(),
                plugin.name.clone(),
                plugin.description.clone(),
                ComponentType::Plugin,
                plugin.priority,
            ),
            metadata: PluginMetadata {
                id: plugin.id,
                name: plugin.name,
                version: plugin.version,
                description: plugin.description,
                author: plugin.author,
                trigger_keywords: plugin.trigger_keywords,
                supported_os: plugin.supported_os,
                priority: plugin.priority,
                // 第三方插件种类（宿主加载时强制覆盖为 ThirdParty，此处显式声明保持语义一致）
                kind: PluginKind::ThirdParty,
                // 全局唤醒热键，仅 panel 形态注册；清单未声明则为 None
                hotkey: plugin.hotkey,
                // panel 形态插件图标由宿主从 manifest [icon] 段读取，此处无需填写
                icon: None,
                mode: plugin.mode,
            },
        }
    }
}

#[async_trait]
impl Configurable for HelloWorldPlugin {
    fn core(&self) -> &ComponentCore {
        &self.core
    }

    /// 本示例无设置项；有设置项时在此声明 schema（label 可用 `t_key!`/`t_key` 形式）。
    fn setting_schema(&self) -> Vec<SettingDefinition> {
        vec![]
    }
}

#[async_trait]
impl Plugin for HelloWorldPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn init(
        &self,
        _ctx: &PluginContext,
        _handle: Option<Arc<PluginHandle>>,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn query(&self, ctx: &PluginContext, query: &Query) -> Result<QueryResponse, PluginError> {
        // ctx.locale 携带宿主当前界面语言（如 "zh-Hans"），可按语言生成本地化文本；
        // 主动查询可用 host().get_locale().await（经 HostProxy）。
        tracing::debug!(locale = %ctx.locale, "hello query");
        Ok(QueryResponse::List {
            results: vec![ListItem {
                id: 1,
                title: format!("Hello: {}", query.raw_query),
                // key-or-literal：前端命中翻译目录则显示译文，否则回退 key 原文。
                // t_key() 自动带当前插件 id 前缀（plugin.com.example.hello-world.<key>）
                subtitle: t_key("greeting"),
                icon: zerolaunch_plugin_api::services::icon_request::IconRequest::Path(String::new()),
                score: 1.0,
                actions: vec![ResultAction {
                    id: "hello".to_string(),
                    label: t_key("sayHello"),
                    icon: zerolaunch_plugin_api::services::icon_request::IconRequest::Path(String::new()),
                    is_default: true,
                    shortcut_key: String::new(),
                }],
                target_type: "BuiltinCommand".to_string(),
                user_arg_count: 0,
                has_system_params: false,
                trigger_keywords: vec![],
            }],
        })
    }

    async fn execute_action(&self, _ctx: &PluginContext, _action_id: &str, _payload: serde_json::Value) -> Result<(), PluginError> {
        tracing::info!("Hello World action executed!");
        Ok(())
    }
}

/// 装配插件应用（骨架 `main()` 调用 `.run()`）。
pub fn app() -> PluginApp {
    PluginApp::new(HelloWorldPlugin::new())
}
