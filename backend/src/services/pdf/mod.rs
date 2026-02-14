// PDF 生成服務 - 拆分為 context (渲染上下文) 和 service (業務邏輯) 子模組
pub mod context;
mod service;

pub use service::*;
