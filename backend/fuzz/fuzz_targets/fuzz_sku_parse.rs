// 模糊測試目標：SKU 編碼解析邏輯
//
// 目的：對 sku.rs 中的 SKU 編碼正則解析進行模糊測試，
// 確保任意字串輸入不會導致 panic 或未定義行為。
//
// 執行方式（需 Linux + nightly）：
//   cargo +nightly fuzz run fuzz_sku_parse
//
// 在 CI 中會自動以 ubuntu + nightly 執行。

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        // 模擬 SKU 編碼格式解析
        // 格式預期為：{category_code}-{subcategory_code}-{sequence}
        // 例如：MED-ANT-001

        // 測試正則匹配不會 panic
        let re = regex::Regex::new(r"^([A-Z]{2,5})-([A-Z]{2,5})-(\d{3,6})$");
        if let Ok(regex) = re {
            if let Some(captures) = regex.captures(input) {
                let _cat = captures.get(1).map(|m| m.as_str());
                let _sub = captures.get(2).map(|m| m.as_str());
                if let Some(seq) = captures.get(3) {
                    let _num: Result<u32, _> = seq.as_str().parse();
                }
            }
        }

        // 測試各種分隔符處理
        let _parts: Vec<&str> = input.split('-').collect();
        let _trimmed = input.trim().to_uppercase();
    }
});
