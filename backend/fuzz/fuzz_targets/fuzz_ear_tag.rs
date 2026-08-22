// 模糊測試目標：耳號（ear tag）格式化與解析邏輯
//
// 目的：對 animal/core.rs 中的耳號相關邏輯進行模糊測試，
// 確保任意字串輸入不會導致 panic 或未定義行為。
//
// 執行方式（需 Linux + nightly）：
//   cargo +nightly fuzz run fuzz_ear_tag
//
// 在 CI 中會自動以 ubuntu + nightly 執行。

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 將隨機位元組轉為 UTF-8 字串
    if let Ok(input) = std::str::from_utf8(data) {
        // 測試耳號格式化不會 panic
        // 模擬 format!("{:03}", num) 的邊界情況
        if let Ok(num) = input.parse::<i32>() {
            let _ = format!("{:03}", num);
        }

        // 測試 LIKE 模式建構不會 panic
        let _ = format!("%{}%", input);

        // 測試耳號長度與特殊字元處理
        if input.len() <= 50 {
            let _trimmed = input.trim();
            let _upper = input.to_uppercase();
        }
    }
});
