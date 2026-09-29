//! 两步验证（2FA/TOTP）登录挂钩：
//!
//! system 模块不依赖 relay 模块（依赖方向约束），因此以 trait + 全局注册表解耦——
//! relay-application 在应用装配时安装实现（见 relay/interface portal_routes 装配点），
//! AuthAppService 在密码校验通过后调用。未安装钩子时行为与原先完全一致。
//!
//! 协议：密码正确但用户启用了 TOTP 且请求未携带验证码 → `AppError::TwoFactorRequired`，
//! 前端收到后要求输入 6 位动态码并随登录参数重提（totpCode），服务端复验后签发令牌。

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use common::error::AppError;

/// 两步验证钩子（由 relay 层实现并注册）
#[async_trait]
pub trait TwoFactorHook: Send + Sync {
    /// 该用户是否已启用两步验证
    async fn required(&self, user_id: &str) -> bool;
    /// 校验 TOTP 动态码（6 位，容忍 ±1 个时间窗）
    async fn verify_code(&self, user_id: &str, code: &str) -> Result<(), AppError>;
}

fn hook_slot() -> &'static OnceLock<Arc<dyn TwoFactorHook>> {
    static S: OnceLock<Arc<dyn TwoFactorHook>> = OnceLock::new();
    &S
}

/// 应用装配时安装钩子（幂等：重复安装忽略后续）
pub fn install_two_factor_hook(hook: Arc<dyn TwoFactorHook>) {
    let _ = hook_slot().set(hook);
}

pub(crate) fn hook() -> Option<Arc<dyn TwoFactorHook>> {
    hook_slot().get().cloned()
}
