-- 027: notification_routing 新增排程頻率欄位
--   每條通知路由規則可設定批次執行的頻率與時間

ALTER TABLE notification_routing
    ADD COLUMN frequency   VARCHAR(20) NOT NULL DEFAULT 'immediate'
        CONSTRAINT chk_nr_frequency CHECK (frequency IN ('immediate','daily','weekly','monthly')),
    ADD COLUMN hour_of_day SMALLINT    NOT NULL DEFAULT 8
        CONSTRAINT chk_nr_hour CHECK (hour_of_day BETWEEN 0 AND 23),
    ADD COLUMN day_of_week SMALLINT    DEFAULT NULL
        CONSTRAINT chk_nr_dow CHECK (day_of_week BETWEEN 0 AND 6);

COMMENT ON COLUMN notification_routing.frequency   IS 'immediate=事件即時觸發, daily/weekly/monthly=批次排程';
COMMENT ON COLUMN notification_routing.hour_of_day IS '批次通知的執行小時（0-23），immediate 時忽略';
COMMENT ON COLUMN notification_routing.day_of_week IS 'weekly 時有效：0=週日, 1=週一 ... 6=週六';

-- 將現有批次型事件改為 daily（保持現有行為：每日執行）
UPDATE notification_routing
    SET frequency = 'daily'
    WHERE event_type IN ('expiry_alert', 'low_stock_alert', 'po_pending_receipt', 'equipment_overdue');
