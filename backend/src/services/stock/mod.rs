mod inventory;
mod ledger;
pub(crate) mod uom;

/// 待通知的儲位帳失真事件。由 `process_document` / `reverse_document_stock` 回傳，
/// 呼叫端在 `tx.commit()` 之後據以通知倉管——寫在 tx 裡會被 rollback 吃掉。
pub use ledger::StorageDriftEvent;

pub struct StockService;
