// 單據服務 - 拆分為 CRUD、工作流、入庫(GRN)、盤點(stocktake) 子模組
mod crud;
mod grn;
mod stocktake;
mod workflow;

pub struct DocumentService;
