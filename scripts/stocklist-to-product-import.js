/**
 * 將「耗材/藥品庫存清表」CSV 轉成「產品匯入範本」格式。
 * 使用方式: node scripts/stocklist-to-product-import.js [輸入檔.csv] [輸出檔.csv]
 * 預設: 20260305_TU-04-00-01_耗材藥品庫存清表(年)_V.0315(TEST).csv -> product_import_from_stocklist.csv
 */

const fs = require('fs');
const path = require('path');

// 解析單行 CSV（支援雙引號包住的欄位）
function parseCSVLine(line) {
  const out = [];
  let cur = '';
  let inQuotes = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (c === '"') {
      if (inQuotes && line[i + 1] === '"') {
        cur += '"';
        i++;
      } else {
        inQuotes = !inQuotes;
      }
    } else if (c === ',' && !inQuotes) {
      out.push(cur.trim());
      cur = '';
    } else {
      cur += c;
    }
  }
  out.push(cur.trim());
  return out;
}

// 品類/子類不在此對照，留空由匯入時系統預設（GEN-OTH）

// 匯出用：將欄位包成 CSV 安全格式（含逗號或換行時加引號）
function escapeCSV(val) {
  if (val == null) return '';
  const s = String(val).trim();
  if (s.includes(',') || s.includes('"') || s.includes('\n') || s.includes('\r')) {
    return '"' + s.replace(/"/g, '""') + '"';
  }
  return s;
}

function main() {
  const root = path.resolve(__dirname, '..');
  const inputPath = path.resolve(root, process.argv[2] || '20260305_TU-04-00-01_耗材藥品庫存清表(年)_V.0315(TEST).csv');
  const outputPath = path.resolve(root, process.argv[3] || 'product_import_from_stocklist.csv');

  const content = fs.readFileSync(inputPath, 'utf8');
  const lines = content.split(/\r?\n/);

  // 表頭在第 4 行 (index 3)，資料從第 5 行 (index 4) 開始
  const DATA_START = 4;
  const headerRow = parseCSVLine(lines[3]);
  // 欄位索引: 請購=0, 存放區=1, 項目=2, 細項=3, 品名=4, 數量=5, 單位=6, 規格=7, 廠商=8, 備註=最後一欄
  const idx = {
    品名: 4,
    數量: 5,
    單位: 6,
    規格: 7,
    廠商: 8,
  };
  const lastColIndex = headerRow.length - 1;

  const outHeader = ['名稱', '規格', '品類代碼', '子類代碼', '單位', '追蹤批號', '追蹤效期', '安全庫存', '備註'];
  const outRows = [outHeader.join(',')];

  let count = 0;
  for (let i = DATA_START; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim()) continue;
    const row = parseCSVLine(line);
    const 品名 = (row[idx.品名] || '').trim();
    if (!品名) continue;

    const 數量 = (row[idx.數量] || '').trim();
    const 單位 = (row[idx.單位] || '').trim() || 'PCS';
    const 規格 = (row[idx.規格] || '').trim();
    const 廠商 = (row[idx.廠商] || '').trim();
    const 備註原 = (row[lastColIndex] || '').trim();
    // 不同廠商分開：有廠商時名稱改為「品名(廠商)」，避免重複判定為同一產品
    const 名稱 = 廠商 ? `${品名}(${廠商})` : 品名;
    const 備註 = 備註原 || (廠商 ? `廠商: ${廠商}` : '');

    const outRow = [
      escapeCSV(名稱),
      escapeCSV(規格),
      '',  // 品類代碼留空，由系統預設 GEN
      '',  // 子類代碼留空，由系統預設 OTH
      escapeCSV(單位),
      'true',
      'true',
      escapeCSV(數量 || ''),
      escapeCSV(備註),
    ];
    outRows.push(outRow.join(','));
    count++;
  }

  fs.writeFileSync(outputPath, '\uFEFF' + outRows.join('\n'), 'utf8'); // BOM for Excel
  console.log(`已寫入 ${count} 筆產品至 ${outputPath}`);
}

main();
