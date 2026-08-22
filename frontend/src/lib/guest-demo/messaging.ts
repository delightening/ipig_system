// Guest demo data — 站內信
//
// 2-3 個範例 thread + 對應 messages，讓 guest 進 /messaging 有東西看。

interface DemoParticipant {
    user_id: string
    display_name: string
    email: string
    last_read_at: string | null
}

interface DemoAttachment {
    id: string
    filename: string
    size: number
    mime_type: string
    created_at: string
}

interface DemoMessage {
    id: string
    thread_id: string
    sender_id: string
    body: string
    created_at: string
    edited_at: string | null
    deleted_at: string | null
    sender_display_name: string
    attachments: DemoAttachment[]
}

interface DemoThreadSummary {
    id: string
    type: 'direct' | 'group'
    subject: string | null
    created_by: string
    created_at: string
    last_message_at: string
    participants: DemoParticipant[]
    unread_count: number
    last_message_preview: string | null
}

interface DemoThreadDetail extends Omit<DemoThreadSummary, 'unread_count' | 'last_message_preview'> {
    messages: DemoMessage[]
}

const VET: DemoParticipant = {
    user_id: 'guest',
    display_name: '訪客',
    email: 'guest@guest.com',
    last_read_at: '2026-05-13T15:00:00Z',
}

const ADMIN: DemoParticipant = {
    user_id: 'demo-u1',
    display_name: '系統管理員',
    email: 'admin@demo.com',
    last_read_at: '2026-05-13T15:00:00Z',
}

const QAU: DemoParticipant = {
    user_id: 'demo-u2',
    display_name: 'QAU 稽核員',
    email: 'qau@demo.com',
    last_read_at: '2026-05-13T14:30:00Z',
}

export const DEMO_THREAD_LIST: DemoThreadSummary[] = [
    {
        id: 'demo-t1',
        type: 'direct',
        subject: '巡場報告複核請確認',
        created_by: 'demo-u2',
        created_at: '2026-05-13T09:00:00Z',
        last_message_at: '2026-05-13T14:25:00Z',
        participants: [VET, QAU],
        unread_count: 1,
        last_message_preview: '已收到本週巡場報告，第三筆觀察建議補上飲水量數據。',
    },
    {
        id: 'demo-t2',
        type: 'group',
        subject: '新批 minipig 進場通知',
        created_by: 'demo-u1',
        created_at: '2026-05-12T10:00:00Z',
        last_message_at: '2026-05-12T11:30:00Z',
        participants: [VET, ADMIN, QAU],
        unread_count: 0,
        last_message_preview: '收到，已安排 D 棟 03 欄位待命。',
    },
    {
        id: 'demo-t3',
        type: 'direct',
        subject: '訓練紀錄繳交提醒',
        created_by: 'demo-u1',
        created_at: '2026-05-10T08:00:00Z',
        last_message_at: '2026-05-10T08:15:00Z',
        participants: [VET, ADMIN],
        unread_count: 0,
        last_message_preview: 'GLP 年度訓練表單請於本月底前繳交，謝謝。',
    },
]

export const DEMO_THREAD_DETAILS: Record<string, DemoThreadDetail> = {
    'demo-t1': {
        id: 'demo-t1',
        type: 'direct',
        subject: '巡場報告複核請確認',
        created_by: 'demo-u2',
        created_at: '2026-05-13T09:00:00Z',
        last_message_at: '2026-05-13T14:25:00Z',
        participants: [VET, QAU],
        messages: [
            {
                id: 'demo-m1',
                thread_id: 'demo-t1',
                sender_id: 'demo-u2',
                body: '請協助複核本週巡場報告第三筆觀察。',
                created_at: '2026-05-13T09:00:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: 'QAU 稽核員',
                attachments: [],
            },
            {
                id: 'demo-m2',
                thread_id: 'demo-t1',
                sender_id: 'guest',
                body: '收到，今天下午會 review。',
                created_at: '2026-05-13T11:20:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: '訪客',
                attachments: [],
            },
            {
                id: 'demo-m3',
                thread_id: 'demo-t1',
                sender_id: 'demo-u2',
                body: '已收到本週巡場報告，第三筆觀察建議補上飲水量數據。',
                created_at: '2026-05-13T14:25:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: 'QAU 稽核員',
                attachments: [],
            },
        ],
    },
    'demo-t2': {
        id: 'demo-t2',
        type: 'group',
        subject: '新批 minipig 進場通知',
        created_by: 'demo-u1',
        created_at: '2026-05-12T10:00:00Z',
        last_message_at: '2026-05-12T11:30:00Z',
        participants: [VET, ADMIN, QAU],
        messages: [
            {
                id: 'demo-m4',
                thread_id: 'demo-t2',
                sender_id: 'demo-u1',
                body: '本週五新批 minipig 5 隻進場，請相關人員注意。',
                created_at: '2026-05-12T10:00:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: '系統管理員',
                attachments: [],
            },
            {
                id: 'demo-m5',
                thread_id: 'demo-t2',
                sender_id: 'guest',
                body: '收到，已安排 D 棟 03 欄位待命。',
                created_at: '2026-05-12T11:30:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: '訪客',
                attachments: [],
            },
        ],
    },
    'demo-t3': {
        id: 'demo-t3',
        type: 'direct',
        subject: '訓練紀錄繳交提醒',
        created_by: 'demo-u1',
        created_at: '2026-05-10T08:00:00Z',
        last_message_at: '2026-05-10T08:15:00Z',
        participants: [VET, ADMIN],
        messages: [
            {
                id: 'demo-m6',
                thread_id: 'demo-t3',
                sender_id: 'demo-u1',
                body: 'GLP 年度訓練表單請於本月底前繳交，謝謝。',
                created_at: '2026-05-10T08:00:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: '系統管理員',
                attachments: [],
            },
            {
                id: 'demo-m7',
                thread_id: 'demo-t3',
                sender_id: 'guest',
                body: '了解，本週內處理。',
                created_at: '2026-05-10T08:15:00Z',
                edited_at: null,
                deleted_at: null,
                sender_display_name: '訪客',
                attachments: [],
            },
        ],
    },
}
