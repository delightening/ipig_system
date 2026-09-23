/// R121：Google 帳號登入（OIDC）。公開模組，供 `handlers/auth/google.rs` 取用型別與常數。
pub mod google_oidc;
mod lockout;
mod login;
mod password;
mod session;
mod two_factor;

#[cfg(test)]
mod tests;

pub struct AuthService;
