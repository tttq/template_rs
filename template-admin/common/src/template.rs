//! 通知模板渲染：把 `${varName}` 占位符替换为变量表里的值
//!
//! 约定：
//! - 占位语法 `${key}`（key 可为字母 / 数字 / 下划线 / 中划线 / 点）
//! - 变量表中不存在该 key 时**原样保留** `${key}`，便于管理员在内容里
//!   一眼看出漏传的变量，而不是被静默替换成空串
//! - 若 `${` 之后没有闭合的 `}`，剩余内容按普通文本输出
//!
//! 业务侧通常配合 [`vars`] 构造变量表：
//!
//! ```ignore
//! common::notify::notify_template(uid, "kyc_approved", template::vars(&[("kycLevel", "2")])).await;
//! ```

use std::collections::HashMap;

/// 渲染模板：将 `${key}` 替换为 `vars[key]`
pub fn render(template: &str, vars: &HashMap<String, String>) -> String {
    // 快速路径：无占位符时直接返回，避免无谓扫描
    if !template.contains("${") {
        return template.to_string();
    }

    let mut out = String::with_capacity(template.len() + 32);
    let mut rest = template;

    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];

        match after.find('}') {
            Some(end) => {
                let key = &after[..end];
                match vars.get(key) {
                    Some(value) => out.push_str(value),
                    // 未传入该变量：保留占位符原文
                    None => {
                        out.push_str("${");
                        out.push_str(key);
                        out.push('}');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                // 缺少闭合括号，剩余内容原样输出
                out.push_str(&rest[start..]);
                return out;
            }
        }
    }

    out.push_str(rest);
    out
}

/// 便捷构造变量表：`vars(&[("userName", "张三"), ("code", "123456")])`
pub fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// 提取模板中出现的全部占位符名（去重，保持出现顺序）
pub fn placeholders(template: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut rest = template;

    while let Some(start) = rest.find("${") {
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let key = &after[..end];
                if !out.iter().any(|k| k == key) {
                    out.push(key.to_string());
                }
                rest = &after[end + 1..];
            }
            None => break,
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_replaces_known_vars() {
        let v = vars(&[("userName", "张三"), ("amount", "100")]);
        assert_eq!(render("你好 ${userName}，到账 ${amount} 元", &v), "你好 张三，到账 100 元");
    }

    #[test]
    fn render_keeps_unknown_placeholder() {
        let v = vars(&[("userName", "张三")]);
        assert_eq!(render("你好 ${userName}，${missing}", &v), "你好 张三，${missing}");
    }

    #[test]
    fn render_handles_no_placeholder() {
        let v = vars(&[]);
        assert_eq!(render("纯文本通知", &v), "纯文本通知");
    }

    #[test]
    fn render_handles_unclosed_placeholder() {
        let v = vars(&[("userName", "张三")]);
        assert_eq!(render("你好 ${userName", &v), "你好 ${userName");
    }

    #[test]
    fn render_is_utf8_safe() {
        let v = vars(&[("name", "值")]);
        assert_eq!(render("【中文】${name}✨", &v), "【中文】值✨");
    }
}
