-- 028: 效期通知範圍設定表 + 月度比較快照表 + fn_expiry_alerts 函數

-- ── 1. 系統層級效期通知設定（全系統單一列） ──────────────────────────────

CREATE TABLE expiry_notification_config (
    id                     UUID     PRIMARY KEY DEFAULT gen_random_uuid(),
    warn_days              SMALLINT NOT NULL DEFAULT 60
        CONSTRAINT chk_enc_warn CHECK (warn_days BETWEEN 1 AND 365),
    cutoff_days            SMALLINT NOT NULL DEFAULT 90
        CONSTRAINT chk_enc_cutoff CHECK (cutoff_days BETWEEN 1 AND 730),
    monthly_threshold_days SMALLINT DEFAULT NULL
        CONSTRAINT chk_enc_monthly CHECK (
            monthly_threshold_days IS NULL OR monthly_threshold_days BETWEEN 1 AND 730
        ),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by             UUID REFERENCES users(id)
);

COMMENT ON TABLE  expiry_notification_config IS '系統層級效期通知範圍設定（全系統僅一列）';
COMMENT ON COLUMN expiry_notification_config.warn_days              IS '提前幾天開始預警（預設 60）';
COMMENT ON COLUMN expiry_notification_config.cutoff_days            IS '過期超過幾天後停止通知（預設 90）';
COMMENT ON COLUMN expiry_notification_config.monthly_threshold_days IS '過期超過此天數後轉月度彙整通知；NULL=停用';

-- 插入預設值（與現有硬編碼一致）
INSERT INTO expiry_notification_config (warn_days, cutoff_days) VALUES (60, 90);

-- ── 2. 月度比較快照（monthly_threshold_days 啟用時使用） ──────────────────

CREATE TABLE expiry_monthly_snapshots (
    id           UUID          PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_ym  CHAR(7)       NOT NULL,       -- 'YYYY-MM'
    product_id   UUID          NOT NULL REFERENCES products(id),
    sku          VARCHAR(80)   NOT NULL,
    product_name VARCHAR(255)  NOT NULL,
    warehouse_id UUID          NOT NULL REFERENCES warehouses(id),
    batch_no     VARCHAR(100),
    expiry_date  DATE          NOT NULL,
    days_past    SMALLINT      NOT NULL,        -- 拍照時距過期天數（正=已過期幾天）
    on_hand_qty  NUMERIC(15,4) NOT NULL,
    base_uom     VARCHAR(20)   NOT NULL,
    created_at   TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX uq_expiry_snapshot_key
    ON expiry_monthly_snapshots (snapshot_ym, product_id, warehouse_id, COALESCE(batch_no, ''));

CREATE INDEX idx_expiry_snapshot_ym ON expiry_monthly_snapshots(snapshot_ym);

COMMENT ON TABLE  expiry_monthly_snapshots IS '每月執行月度效期通知時的品項快照，供下月比對新增/減少';
COMMENT ON COLUMN expiry_monthly_snapshots.snapshot_ym IS '快照月份，格式 YYYY-MM';
COMMENT ON COLUMN expiry_monthly_snapshots.days_past   IS '拍照時距效期已過幾天（負值=尚未過期）';

-- ── 3. fn_expiry_alerts：可傳入動態參數的效期查詢函數 ────────────────────
--   取代 v_expiry_alerts 視圖用於排程器（視圖保留供 UI 直接查詢用）

CREATE OR REPLACE FUNCTION fn_expiry_alerts(
    p_warn_days   INT DEFAULT 60,
    p_cutoff_days INT DEFAULT 90
)
RETURNS TABLE (
    product_id        UUID,
    sku               VARCHAR,
    product_name      VARCHAR,
    spec              TEXT,
    category_code     VARCHAR,
    warehouse_id      UUID,
    warehouse_code    VARCHAR,
    warehouse_name    VARCHAR,
    batch_no          VARCHAR,
    expiry_date       DATE,
    on_hand_qty       NUMERIC,
    base_uom          VARCHAR,
    days_until_expiry INT,
    expiry_status     VARCHAR,
    total_qty         NUMERIC
)
LANGUAGE SQL STABLE AS $$
    SELECT
        p.id                                                              AS product_id,
        p.sku,
        p.name                                                            AS product_name,
        p.spec,
        p.category_code,
        sl.warehouse_id,
        w.code                                                            AS warehouse_code,
        w.name                                                            AS warehouse_name,
        sl.batch_no,
        sl.expiry_date,
        SUM(CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            ELSE -sl.qty_base
        END)                                                              AS on_hand_qty,
        p.base_uom,
        (sl.expiry_date - CURRENT_DATE)::INT                             AS days_until_expiry,
        CASE WHEN sl.expiry_date < CURRENT_DATE
             THEN 'expired'
             ELSE 'expiring_soon'
        END                                                               AS expiry_status,
        COALESCE(inv.on_hand_qty_base, 0)                                AS total_qty
    FROM stock_ledger sl
    JOIN products p    ON sl.product_id   = p.id
    JOIN warehouses w  ON sl.warehouse_id = w.id
    LEFT JOIN inventory_snapshots inv
           ON inv.product_id = p.id AND inv.warehouse_id = sl.warehouse_id
    WHERE p.track_expiry = true
      AND sl.expiry_date IS NOT NULL
      AND p.is_active    = true
      AND sl.expiry_date >= CURRENT_DATE - p_cutoff_days
    GROUP BY p.id, p.sku, p.name, p.spec, p.category_code,
             sl.warehouse_id, w.code, w.name, sl.batch_no, sl.expiry_date,
             p.base_uom, inv.on_hand_qty_base
    HAVING
        SUM(CASE
            WHEN sl.direction IN ('in', 'transfer_in', 'adjust_in') THEN sl.qty_base
            ELSE -sl.qty_base
        END) > 0
        AND sl.expiry_date <= CURRENT_DATE + p_warn_days
$$;

COMMENT ON FUNCTION fn_expiry_alerts IS '效期預警查詢函數，支援動態傳入提前預警天數與截止天數';
