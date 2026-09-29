//! 邮箱验证码 / 图形验证码应用服务
//!
//! 背景：登录（`loginType = email_code`）、注册、改密码、换邮箱都需要「邮箱验证码」，
//! 而发码必须先过图形验证码防止被脚本刷。本服务把发码侧（图形码生成 / 校验、
//! 冷却与日限、验证码落 Redis、按通知模板投递邮件）统一收在 system 模块内，
//! 与 `AuthAppService` 的消费侧（`verify_email_code`）共享同一 Redis key 约定
//! （`common::email_code_redis_key`）。
//!
//! Redis key 约定：
//! - `captcha:img:{id}`   图形码答案（TTL 300s，一次性消费）
//! - `mail:code:{email}:{scene}`  邮箱验证码（TTL 600s，一次性消费）
//! - `mail:cooldown:{email}`      发码冷却（60s）
//! - `mail:day:ip:{ip}:{yyyymmdd}` / `mail:day:email:{email}:{yyyymmdd}` 日限

use common::error::AppError;
use common::generate_email_code;
use summer::plugin::service::Service;
use summer_redis::Redis;
use summer_redis::redis;
use summer_mail::config::MailerConfig;

use super::captcha;
use super::dto::{CaptchaVo, SendCodeDto};

/// 邮箱验证码有效时长（秒）
const CODE_TTL_SECS: u64 = 600; // 10min
/// 同一邮箱发码冷却（秒）
const CODE_COOLDOWN_SECS: u64 = 60;
/// 图形验证码有效期（5min）
const CAPTCHA_TTL_SECS: u64 = 300;
/// 单 IP 每分钟图形码获取上限
const CAPTCHA_MINUTE_LIMIT: i64 = 20;
/// 单 IP 每日图形码获取上限
const CAPTCHA_DAY_LIMIT: i64 = 500;

/// 邮箱验证码 Redis key（与 `AuthAppService` 的校验端共享同一约定）
fn code_key(email: &str, scene: &str) -> String {
    common::email_code_redis_key(email, scene)
}

fn cooldown_key(email: &str) -> String {
    format!("mail:cooldown:{}", email)
}

fn daily_key(kind: &str, ident: &str) -> String {
    format!(
        "mail:day:{}:{}:{}",
        kind,
        ident,
        chrono::Utc::now().format("%Y%m%d")
    )
}

#[derive(Clone, Service)]
pub struct AuthCodeService {
    #[inject(component)]
    redis: Redis,
    /// 邮件配置（`stub` 开关用于本地联调打印验证码；
    /// 真实投递由通知中心按通知模板渲染后统一发送，见 common::notify::send_email_template）
    #[inject(config)]
    mail_cfg: MailerConfig,
}

impl AuthCodeService {
    /// 生成图形验证码：按 IP 限速，答案写入 Redis（TTL 300s），返回 data URI 供前端展示
    pub async fn generate_captcha(&self, ip: Option<&str>) -> Result<CaptchaVo, AppError> {
        let ip = ip.unwrap_or("unknown").to_string();
        let mut conn = self.redis.clone();

        // 分钟级获取上限（防脚本刷图）
        let min_key = captcha::minute_limit_key(&ip);
        let n: i64 = redis::cmd("INCR")
            .arg(&min_key)
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;
        if n == 1 {
            let _ = redis::cmd("EXPIRE")
                .arg(&min_key)
                .arg(60)
                .query_async::<i64>(&mut conn)
                .await;
        }
        if n > CAPTCHA_MINUTE_LIMIT {
            return Err(AppError::BadRequest("@captcha_too_frequent".to_string()));
        }

        // 日级获取上限
        let day_key = captcha::day_limit_key(&ip);
        let d: i64 = redis::cmd("INCR")
            .arg(&day_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);
        if d == 1 {
            let _ = redis::cmd("EXPIRE")
                .arg(&day_key)
                .arg(86400)
                .query_async::<i64>(&mut conn)
                .await;
        }
        if d > CAPTCHA_DAY_LIMIT {
            return Err(AppError::BadRequest("@captcha_daily_limit".to_string()));
        }

        let code = captcha::random_captcha_code();
        let id = captcha::random_captcha_id();
        redis::cmd("SET")
            .arg(captcha::captcha_id_key(&id))
            .arg(&code)
            .arg("EX")
            .arg(CAPTCHA_TTL_SECS)
            .query_async::<String>(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;

        let png = captcha::render_png(&code)?;
        Ok(CaptchaVo {
            captcha_id: id,
            image: captcha::png_data_uri(&png),
        })
    }

    /// 图形码 PNG 原图（服务端按已存答案重绘，供小程序 <image> 按 URL 加载）
    pub async fn captcha_png(&self, id: &str) -> Result<Vec<u8>, AppError> {
        let mut conn = self.redis.clone();
        let code: Option<String> = redis::cmd("GET")
            .arg(captcha::captcha_id_key(id))
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_down".to_string()))?;
        match code {
            Some(c) => captcha::render_png(&c),
            None => Err(AppError::NotFound("@captcha_not_found".to_string())),
        }
    }

    /// 校验并消费图形验证码（一次性：无论对错均删除，避免在同一 id 上爆破重试）
    async fn consume_captcha(&self, id: &str, code: &str) -> Result<(), AppError> {
        if id.trim().is_empty() || code.trim().is_empty() {
            return Err(AppError::BadRequest("@captcha_required".to_string()));
        }
        let mut conn = self.redis.clone();
        let key = captcha::captcha_id_key(id);
        let saved: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_down".to_string()))?;
        let matched = saved
            .as_deref()
            .map(|s| s.eq_ignore_ascii_case(code.trim()))
            .unwrap_or(false);
        let _ = redis::cmd("DEL")
            .arg(&key)
            .query_async::<i64>(&mut conn)
            .await;
        if matched {
            Ok(())
        } else {
            Err(AppError::BadRequest("@captcha_invalid".to_string()))
        }
    }

    /// 发送邮箱验证码：先校验图形验证码，再走冷却 60s、日限（fail-close，Redis 故障直接拒绝）
    pub async fn send_code(&self, dto: SendCodeDto, ip: Option<&str>) -> Result<(), AppError> {
        // scene 白名单：register=注册/完善资料、reset=找回密码、login=邮箱验证码登录、
        // change_password/change_email=已登录用户的账户安全操作（改密码/换邮箱）
        if !matches!(
            dto.scene.as_str(),
            "register" | "reset" | "login" | "change_password" | "change_email"
        ) {
            return Err(AppError::BadRequest("@captcha_scene_invalid".to_string()));
        }
        if !dto.email.contains('@') || dto.email.len() > 100 {
            return Err(AppError::BadRequest("@email_invalid".to_string()));
        }
        // 图形验证码（一次性）优先校验，校验失败不占用下方邮箱冷却
        self.consume_captcha(&dto.captcha_id, &dto.captcha_code).await?;
        let mut conn = self.redis.clone();

        // 冷却检查（SET NX EX）
        let cooled = redis::cmd("SET")
            .arg(cooldown_key(&dto.email))
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(CODE_COOLDOWN_SECS)
            .query_async::<Option<String>>(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;
        if cooled.is_none() {
            return Err(AppError::BadRequest("@send_too_frequent".to_string()));
        }

        // 日限检查（fail-close）
        let ip = ip.unwrap_or("unknown").to_string();
        for (kind, ident, limit) in [("ip", ip, 10i64), ("email", dto.email.clone(), 5i64)] {
            let n: i64 = redis::cmd("INCR")
                .arg(daily_key(kind, &ident))
                .query_async(&mut conn)
                .await
                .unwrap_or(0);
            if n == 1 {
                let _ = redis::cmd("EXPIRE")
                    .arg(daily_key(kind, &ident))
                    .arg(86400)
                    .query_async::<i64>(&mut conn)
                    .await;
            }
            if n > limit {
                return Err(AppError::BadRequest("@email_code_daily_limit".to_string()));
            }
        }

        let code = generate_email_code();
        redis::cmd("SET")
            .arg(code_key(&dto.email, &dto.scene))
            .arg(&code)
            .arg("EX")
            .arg(CODE_TTL_SECS)
            .query_async::<String>(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;

        // 邮件标题与正文由通知模板渲染
        // （email_code_register / email_code_reset / email_code_login
        //   / email_code_change_password / email_code_change_email），
        // 管理员可在「系统管理 → 通知模板」中直接调整文案与样式，无需改代码。
        // stub 模式（本地联调）仅打印验证码，不真实投递。
        if self.mail_cfg.stub {
            log::info!("[mail:{}] {} 验证码: {}", dto.scene, dto.email, code);
        } else {
            let template_code = match dto.scene.as_str() {
                "reset" => "email_code_reset",
                "login" => "email_code_login",
                "change_password" => "email_code_change_password",
                "change_email" => "email_code_change_email",
                _ => "email_code_register",
            };
            common::notify::send_email_template(
                &dto.email,
                template_code,
                common::template::vars(&[("code", &code)]),
            )
            .await?;
        }
        Ok(())
    }
}