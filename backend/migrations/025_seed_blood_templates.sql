-- 從豬博士動物科技生化檢驗項目表匯入所有檢驗模板
-- ON CONFLICT DO UPDATE 確保冪等

INSERT INTO blood_test_templates (code, name, default_unit, sort_order) VALUES
-- 肝 (Liver)
('T-BIL',  'T. Bilirubin (總膽紅素)',      'mg/dL',  101),
('D-BIL',  'D. Bilirubin (直接膽紅素)',     'mg/dL',  102),
('AST',    'AST (GOT)',                     'U/L',    103),
('ALT',    'ALT (GPT)',                     'U/L',    104),
('ALP',    'ALP (鹼性磷酸酶)',              'U/L',    105),
('GGT',    'γ-GT',                          'U/L',    106),
('TP',     'Total Protein (總蛋白)',        'g/dL',   107),
('ALB',    'ALB (白蛋白)',                  'g/dL',   108),
('GLO',    'GLO (球蛋白)',                  'g/dL',   109),
('AG',     'A/G (白球比)',                  NULL,     110),
-- 主脂 (Lipids)
('CHO',    'CHO (膽固醇)',                  'mg/dL',  201),
('TG',     'TG (三酸甘油酯)',              'mg/dL',  202),
('HDL',    'HDL',                           'mg/dL',  203),
('LDL',    'LDL',                           'mg/dL',  204),
-- 心臟 (Heart)
('CK',     'CK (肌酸激酶)',                'U/L',    301),
('LDH',    'LDH (乳酸脫氫酶)',             'U/L',    302),
('HSCRP',  'hs-CRP (高敏C反應蛋白)',       'mg/L',   303),
('CKMB',   'CK-MB',                        'U/L',    304),
-- 腎臟/胰 (Pancreas)
('AMY',    'Amylase (澱粉酶)',             'U/L',    401),
('LPS',    'Lipase (脂肪酶)',              'U/L',    402),
-- 醣/胰 (Sugar/Pancreas)
('GLU',    'Glucose (血糖)',               'mg/dL',  501),
('HBA1C',  'HbA1c (糖化血色素)',           '%',      502),
('INS',    'Insulin (胰島素)',             'μU/mL',  503),
('CPEP',   'C-Peptide (C-胜肽)',           'ng/mL',  504),
-- 腎 (Kidney)
('BUN',    'BUN (血尿素氮)',               'mg/dL',  601),
('CRE',    'CRE (肌酐酸)',                 'mg/dL',  602),
('UA',     'UA (尿酸)',                    'mg/dL',  603),
-- 泌尿 (Urinary)
('UREA',   'Urea Nitrogen (尿素氮)',       'mg/dL',  701),
('UTP',    'UTP (尿蛋白定量)',             'mg/day', 702),
('MALB',   'Microalbumin (微白蛋白)',      'mg/L',   703),
('URINE',  'Urine routine (尿液常規)',     NULL,     704),
-- 血液/凝血 (Coagulation)
('PT',     'PT (凝血酶原時間)',            'sec',    801),
('APTT',   'APTT (活化部分凝血時間)',      'sec',    802),
('FIB',    'Fibrinogen (纖維蛋白原)',      'mg/dL',  803),
('DDIM',   'D-Dimer (D-二聚體)',           'μg/mL',  804),
('ESR',    'ESR (紅血球沉降速率)',         'mm/hr',  805),
-- 電解質 (Electrolytes)
('NA',     'Na+ (鈉)',                     'mEq/L',  901),
('K',      'K+ (鉀)',                      'mEq/L',  902),
('CL',     'Cl- (氯)',                     'mEq/L',  903),
('CA',     'Ca2+ (鈣)',                    'mg/dL',  904),
('MG',     'Mg2+ (鎂)',                    'mg/dL',  905),
('PHOS',   'P (磷)',                       'mg/dL',  906),
-- 荷爾蒙 (Hormones)
('PRL',    'Prolactine (泌乳素)',          'ng/mL',  1001),
('TESTO',  'Testosterone (睪固酮)',        'ng/dL',  1002),
('FSH',    'FSH (濾泡刺激素)',             'mIU/mL', 1003),
('LH',     'LH (黃體生成素)',              'mIU/mL', 1004),
('PROG',   'Progesterone (黃體素)',        'ng/mL',  1005),
('E2',     'E2 (雌二醇)',                  'pg/mL',  1006),
('AE3',    'αE3',                          NULL,     1007),
-- 其他 (Others)
('LAC',    'Lactate (乳酸)',               'mmol/L', 1101),
('CTX',    'CTx (骨膠原交聯)',             'ng/mL',  1102),
('HBANAL', 'Hb analysis (血色素分析)',     NULL,     1103),
-- 感染 (Infection)
('AFB',    '抗酸菌',                       NULL,     1201),
('IGRA',   'IGRA (干擾素測試)',            NULL,     1202),
('CULT',   '沿革 & 鑑定',                  NULL,     1203),
-- CBC 基礎（CBC 為集合名稱，不列為獨立項目）
('WBC',    'WBC (白血球計數)',             '10³/μL', 2),
('RBC',    'RBC (紅血球計數)',             '10⁶/μL', 3),
('HGB',    'HGB (血紅素)',                 'g/dL',   4),
('HCT',    'HCT (血容比)',                 '%',      5),
('PLT',    'PLT (血小板計數)',             '10³/μL', 6),
-- 採血管 (Blood Collection Tubes)
('EDTA',   'EDTA 採血管',                  '支',     1301),
('HEP',    'Heparin 採血管',               '支',     1302),
('SST',    'SST 採血管',                   '支',     1303),
('SCIT',   'Sodium Citrate 採血管',        '支',     1304)
ON CONFLICT (code) DO UPDATE SET
  name = EXCLUDED.name,
  default_unit = EXCLUDED.default_unit,
  sort_order = EXCLUDED.sort_order;
