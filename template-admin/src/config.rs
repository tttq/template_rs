use summer_sa_token::{PathAuthBuilder, SaTokenConfigurator};

pub struct SaTokenConfig;

impl SaTokenConfigurator for SaTokenConfig {
    fn configure_path_auth(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
        auth.include("/api/**")
            .exclude("/api/auth/login")
            .exclude("/api/auth/captcha")
            .exclude("/api/auth/register")
            .exclude("/api/auth/locate")
            .exclude("/api/auth/refresh-token")
            .exclude("/api/auth/providers")
    }
}
