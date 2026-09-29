use summer_sa_token::{PathAuthBuilder, SaTokenConfigurator};

pub struct SaTokenConfig;

impl SaTokenConfigurator for SaTokenConfig {
    fn configure_path_auth(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
        auth.include("/api/**")
            .exclude("/api/auth/login")
            .exclude("/api/auth/register")
            .exclude("/api/auth/register-roles")
            .exclude("/api/auth/locate")
            .exclude("/api/auth/refresh-token")
            .exclude("/api/auth/providers")
            // 开放认证端点（图形验证码 / 邮箱验证码 / 微信扫码地址）：
            // handler 上的 #[sa_ignore] 不参与路径认证判断，必须在此显式豁免，否则未登录全部 401
            .exclude("/api/auth/captcha")
            .exclude("/api/auth/captcha-image/**")
            .exclude("/api/auth/send-code")
            .exclude("/api/auth/wechat/qr-url")
            // 健康检查
            .exclude("/api/health")
    }
}