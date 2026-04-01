import type { ProtocolListItem } from '@/types/aup'

export const DEMO_PROTOCOLS: ProtocolListItem[] = [
  {
    id: 'demo-p1', protocol_no: 'AUP-2025-001', iacuc_no: 'IACUC-2025-001',
    title: '範例研究計畫：心血管藥物安全性評估', status: 'APPROVED',
    pi_user_id: 'demo-u1', pi_name: '範例研究員 A',
    start_date: '2025-06-01', end_date: '2026-12-31',
    created_at: '2025-05-15T08:00:00Z', apply_study_number: 'STD-2025-001',
  },
  {
    id: 'demo-p2', protocol_no: 'AUP-2025-002', iacuc_no: 'IACUC-2025-002',
    title: '範例研究計畫：骨科植入物生物相容性試驗', status: 'UNDER_REVIEW',
    pi_user_id: 'demo-u2', pi_name: '範例研究員 B',
    start_date: '2026-01-01', end_date: '2027-06-30',
    created_at: '2025-11-20T08:00:00Z', apply_study_number: 'STD-2025-002',
  },
  {
    id: 'demo-p3', protocol_no: 'AUP-2026-001',
    title: '範例研究計畫：新型疫苗免疫反應研究', status: 'DRAFT',
    pi_user_id: 'demo-u3', pi_name: '範例研究員 C',
    created_at: '2026-03-01T08:00:00Z',
  },
]
