-- 024: 新增 MCP_READ 活動類型（透過 MCP 工具閱覽計畫書的稽核紀錄）
ALTER TYPE protocol_activity_type ADD VALUE IF NOT EXISTS 'MCP_READ';
