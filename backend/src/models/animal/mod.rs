// 動物管理模型 - 拆分為 enums、entities、requests 三個子模組
pub mod enums;
pub mod entities;
pub mod requests;

// Re-export 所有公開型別以維持相容性
pub use enums::*;
pub use entities::*;
pub use requests::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // AnimalStatus 測試
    // ==========================================

    #[test]
    fn test_animal_status_display_name() {
        assert_eq!(AnimalStatus::Unassigned.display_name(), "未分配");
        assert_eq!(AnimalStatus::InExperiment.display_name(), "實驗中");
        assert_eq!(AnimalStatus::Completed.display_name(), "實驗完成");
    }

    #[test]
    fn test_animal_status_serde() {
        assert_eq!(serde_json::to_string(&AnimalStatus::InExperiment).unwrap(), "\"in_experiment\"");
        let status: AnimalStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(status, AnimalStatus::Completed);
    }

    // ==========================================
    // AnimalBreed 測試
    // ==========================================

    #[test]
    fn test_animal_breed_display_name() {
        assert_eq!(AnimalBreed::Minipig.display_name(), "迷你豬");
        assert_eq!(AnimalBreed::White.display_name(), "白豬");
        assert_eq!(AnimalBreed::LYD.display_name(), "LYD");
        assert_eq!(AnimalBreed::Other.display_name(), "其他");
    }

    #[test]
    fn test_animal_breed_serde() {
        // 前端使用 "minipig" 而非 "miniature"
        assert_eq!(serde_json::to_string(&AnimalBreed::Minipig).unwrap(), "\"minipig\"");
        assert_eq!(serde_json::to_string(&AnimalBreed::LYD).unwrap(), "\"lyd\"");

        let breed: AnimalBreed = serde_json::from_str("\"minipig\"").unwrap();
        assert_eq!(breed, AnimalBreed::Minipig);
    }

    // ==========================================
    // AnimalGender 測試
    // ==========================================

    #[test]
    fn test_animal_gender_display_name() {
        assert_eq!(AnimalGender::Male.display_name(), "公");
        assert_eq!(AnimalGender::Female.display_name(), "母");
    }

    #[test]
    fn test_animal_gender_serde() {
        assert_eq!(serde_json::to_string(&AnimalGender::Male).unwrap(), "\"male\"");
        let gender: AnimalGender = serde_json::from_str("\"female\"").unwrap();
        assert_eq!(gender, AnimalGender::Female);
    }

    // ==========================================
    // RecordType 測試
    // ==========================================

    #[test]
    fn test_record_type_display_name() {
        assert_eq!(RecordType::Abnormal.display_name(), "異常紀錄");
        assert_eq!(RecordType::Experiment.display_name(), "試驗紀錄");
        assert_eq!(RecordType::Observation.display_name(), "觀察紀錄");
    }

    // ==========================================
    // validate_ear_tag 測試
    // ==========================================

    #[test]
    fn test_validate_ear_tag_three_digits() {
        assert!(validate_ear_tag("001").is_ok());
        assert!(validate_ear_tag("999").is_ok());
    }

    #[test]
    fn test_validate_ear_tag_numeric_formatting() {
        // 數字 "1" 會自動格式化為 "001"
        assert!(validate_ear_tag("1").is_ok());
        assert!(validate_ear_tag("12").is_ok());
    }

    #[test]
    fn test_validate_ear_tag_invalid() {
        assert!(validate_ear_tag("1234").is_err()); // 超過三位
        assert!(validate_ear_tag("abc").is_err());  // 非數字
    }

    // ==========================================
    // validate_pen_location 測試
    // ==========================================

    #[test]
    fn test_validate_pen_location_valid() {
        assert!(validate_pen_location(&"A-01".to_string()).is_ok());
    }

    #[test]
    fn test_validate_pen_location_empty() {
        assert!(validate_pen_location(&"".to_string()).is_err());
        assert!(validate_pen_location(&"   ".to_string()).is_err()); // 空白也不行
    }
}
