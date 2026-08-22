import type { ProtocolListItem, ProtocolResponse } from '@/types/aup'
import type { ProtocolWorkingContent } from '@/types/protocol'

// 註：demo working_content 故意以 `as unknown as ProtocolWorkingContent` 雙重 cast。
// ProtocolWorkingContent 是嚴格 Zod schema（含許多必填欄位），demo 資料只填觀眾會看到的
// 顯示欄位、結構也不完全等同（例如 surgery 欄位形狀近似實際 API response 但欄位名 / 型別
// 略有差異）。Partial<T> / DemoWorkingContent 等較鬆的 helper type 都無法直接 assign
// 給 ProtocolResponse.working_content（schema 期望嚴格 T），因此採用 unknown bridge。
// 詳情頁皆以 optional chaining 存取，runtime 沒風險；schema 演進時人工同步即可。

// 故事線時序：
// AUP-2025-001（D-001/D-002 試驗計畫）：撰寫 9 月中 → 提交 9-20 → 審查 5 週 →
//   核准 11-01 → 動物進場 11-15 → 實驗 2026-01 起 → 預計結案 2026-12
// AUP-2025-002（D-005 計畫）：審核中
// AUP-2026-001（demo-p3）：DRAFT 草稿
export const DEMO_PROTOCOLS: ProtocolListItem[] = [
  {
    id: 'demo-p1', protocol_no: 'AUP-2025-001', iacuc_no: 'IACUC-2025-001',
    title: '範例研究計畫：心血管藥物安全性評估', status: 'APPROVED',
    pi_user_id: 'demo-u1', pi_name: '範例研究員 A',
    // 計畫核准日 = 動物開始實驗的最早可能日；start = 2026-01-05 進入實驗
    start_date: '2026-01-05', end_date: '2026-12-31',
    created_at: '2025-09-15T08:00:00Z', apply_study_number: 'STD-2025-001',
  },
  {
    id: 'demo-p2', protocol_no: 'AUP-2025-002', iacuc_no: 'IACUC-2025-002',
    title: '範例研究計畫：骨科植入物生物相容性試驗', status: 'UNDER_REVIEW',
    pi_user_id: 'demo-u2', pi_name: '範例研究員 B',
    start_date: '2026-04-01', end_date: '2027-06-30',
    created_at: '2026-02-20T08:00:00Z', apply_study_number: 'STD-2025-002',
  },
  {
    id: 'demo-p3', protocol_no: 'AUP-2026-001',
    title: '範例研究計畫：新型疫苗免疫反應研究', status: 'DRAFT',
    pi_user_id: 'demo-u3', pi_name: '範例研究員 C',
    created_at: '2026-03-15T08:00:00Z',
  },
]

// ============================================
// Detail 頁完整 ProtocolResponse（含 working_content 各 section）
// ============================================

const SHARED_PI = {
  pi_name: '範例研究員 A',
  pi_email: 'pi@example.com',
  pi_organization: '範例研究機構 / 心血管研究所',
}

export const DEMO_PROTOCOL_DETAIL_P1: ProtocolResponse = {
  protocol: {
    id: 'demo-p1',
    protocol_no: 'AUP-2025-001',
    iacuc_no: 'IACUC-2025-001',
    title: '範例研究計畫：心血管藥物安全性評估',
    status: 'APPROVED',
    pi_user_id: 'demo-u1',
    start_date: '2026-01-05',
    end_date: '2026-12-31',
    created_by: 'demo-u1',
    created_at: '2025-09-15T08:00:00Z',
    updated_at: '2025-11-01T14:00:00Z',
    version: 3,
    // Demo working_content：用 loose type cast 避免完整 schema 約束（Detail 頁
     // 用 optional chaining 取值，缺欄位顯示為空）
     working_content: {
      basic: {
        apply_study_number: 'STD-2025-001',
        study_title: '心血管藥物安全性評估 — 迷你豬模型',
        execution_period: { start: '2026-01-05', end: '2026-12-31' },
        pi: {
          name: '範例研究員 A',
          email: 'pi@example.com',
          organization: '範例研究機構 / 心血管研究所',
          phone: '範例電話',
          address: '範例地址',
        },
        funding_source: '範例科研補助計畫',
      },
      purpose: {
        abstract: '本計畫旨在評估新型抗血栓藥物（範例代號 NTC-001）於迷你豬模型之 28 天重複給藥安全性，特別觀察心血管系統的血液動力學影響、血液凝固指標變化及主要器官病理變化。',
        significance: '迷你豬與人類心血管系統解剖、生理高度相似，是國際法規（ICH S7）認可的非囓齒類安全性評估動物模型。',
        objectives: [
          '評估 NTC-001 於 28 天重複口服給藥之耐受性',
          '監測心率、血壓、心電圖等心血管參數變化',
          '量測血液凝固相關生化指標（PT、aPTT、INR）',
          '主要器官（心、肝、腎）病理組織學評估',
        ],
      },
      design: {
        groups: [
          { name: '對照組', count: 2, dose: '生理食鹽水 0.5 mL/kg', route: 'PO', frequency: 'QD × 28 天' },
          { name: '低劑量組', count: 2, dose: 'NTC-001 5 mg/kg', route: 'PO', frequency: 'QD × 28 天' },
          { name: '高劑量組', count: 2, dose: 'NTC-001 25 mg/kg', route: 'PO', frequency: 'QD × 28 天' },
        ],
        observation_schedule: '給藥前 7 天、給藥期間每日臨床觀察、第 7/14/21/28 天採血生化、第 28 天結束時心電圖 + 血液動力學量測 + 病理解剖',
        expected_end_point: '完成 28 天給藥後人道犧牲；體重下降 > 20%、嚴重不良反應即提早終止',
      },
      animals: {
        animals: [
          { ear_tag: 'D-001', breed: 'minipig', gender: 'female', count: 1 },
          { ear_tag: 'D-002', breed: 'minipig', gender: 'male', count: 1 },
        ],
        total_animals: 6,
        notes: '已進場 2 隻，預計再進場 4 隻補足樣本數',
      },
      personnel: [
        { name: '範例研究員 A', roles: ['PI'], training: ['動物實驗倫理 6 學分', 'GLP 訓練'] },
        { name: '範例獸醫', roles: ['VET'], training: ['獸醫師執照', '迷你豬麻醉訓練'] },
        { name: '範例實驗員', roles: ['EXPERIMENT_STAFF'], training: ['動物實驗倫理 3 學分'] },
      ],
      anesthesia: {
        induction: 'Atropine 0.05 mg/kg IM + Stroless 5 mg/kg IM',
        maintenance: 'Isoflurane 1.5–2% in 100% O₂',
        monitoring: '心電圖 / SpO₂ / 直腸體溫 / 末梢血氧',
      },
      surgery: {
        surgery_type: '頸動脈導管置入',
        surgery_description: '心臟採血（左頸動脈導管）+ 部分動物進行頸動脈分離訓練（不切開）',
        preop_preparation: '禁食 12 小時、術前抗生素預防 (Cefazolin 20 mg/kg IV)',
      },
      end_handling: {
        method: '人道犧牲（深度麻醉 + 過量 KCl IV）',
        carcass_disposal: '依 SOP-FAC-002 凍存後委託合格廢棄物處理廠處理',
      },
    } as unknown as ProtocolWorkingContent,
  },
  pi_name: SHARED_PI.pi_name,
  pi_email: SHARED_PI.pi_email,
  pi_organization: SHARED_PI.pi_organization,
  status_display: '已通過',
  vet_review: undefined,
}

export const DEMO_PROTOCOL_DETAIL_P2: ProtocolResponse = {
  protocol: {
    id: 'demo-p2',
    protocol_no: 'AUP-2025-002',
    iacuc_no: 'IACUC-2025-002',
    title: '範例研究計畫：骨科植入物生物相容性試驗',
    status: 'UNDER_REVIEW',
    pi_user_id: 'demo-u2',
    start_date: '2026-04-01',
    end_date: '2027-06-30',
    created_by: 'demo-u2',
    created_at: '2026-02-20T08:00:00Z',
    updated_at: '2026-04-10T08:00:00Z',
    version: 1,
    working_content: {
      basic: {
        apply_study_number: 'STD-2025-002',
        study_title: '新型鈦合金植入物生物相容性 6 個月評估',
        execution_period: { start: '2026-04-01', end: '2027-06-30' },
        pi: { name: '範例研究員 B', email: 'pi-b@example.com', organization: '範例骨科研究中心', phone: '範例電話', address: '範例地址' },
      },
      purpose: {
        abstract: '評估新型鈦合金（範例代號 TI-CHM）於迷你豬股骨植入後 6 個月之骨整合反應、組織病理及血清離子濃度變化。',
        objectives: ['骨整合 X 光與 micro-CT 影像評估', '血清離子（Ti / Al / V）濃度監測', '植入部位組織學切片'],
      },
    } as unknown as ProtocolWorkingContent,
  },
  pi_name: '範例研究員 B',
  status_display: '審核中',
}

export const DEMO_PROTOCOL_DETAIL_P3: ProtocolResponse = {
  protocol: {
    id: 'demo-p3',
    protocol_no: 'AUP-2026-001',
    title: '範例研究計畫：新型疫苗免疫反應研究',
    status: 'DRAFT',
    pi_user_id: 'demo-u3',
    created_by: 'demo-u3',
    created_at: '2026-03-15T08:00:00Z',
    updated_at: '2026-03-15T08:00:00Z',
    version: 1,
    working_content: {
      basic: {
        apply_study_number: 'STD-2026-001',
        study_title: '草稿中 — 細節撰寫中',
        pi: { name: '範例研究員 C', email: 'pi-c@example.com', phone: '範例電話', address: '範例地址' },
      },
    } as unknown as ProtocolWorkingContent,
  },
  pi_name: '範例研究員 C',
  status_display: '草稿',
}
