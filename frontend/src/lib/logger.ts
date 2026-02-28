/**
 * 日誌封裝：開發環境輸出至 console，生產環境靜默
 * P0-R4-3：避免生產環境洩漏內部狀態、API 回應、錯誤堆疊
 */
const isDev = import.meta.env.DEV;

export const logger = {
  log: (...args: unknown[]) => {
    if (isDev) console.log(...args);
  },
  warn: (...args: unknown[]) => {
    if (isDev) console.warn(...args);
  },
  error: (...args: unknown[]) => {
    if (isDev) console.error(...args);
  },
};
