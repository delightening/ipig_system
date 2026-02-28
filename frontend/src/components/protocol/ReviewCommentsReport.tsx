import { formatDate } from '@/lib/utils'
import { useRef } from 'react'
import type { Protocol, ReviewCommentResponse, VetReviewAssignment, VetReviewItem } from '@/types/aup'

interface ReviewCommentsReportProps {
    protocol: Protocol
    comments: ReviewCommentResponse[]
    vet_review?: VetReviewAssignment
}

export function ReviewCommentsReport({ protocol, comments, vet_review }: ReviewCommentsReportProps) {
    const reportRef = useRef<HTMLDivElement>(null)

    const basic = protocol?.working_content?.basic

    const getReplies = (commentId: string) => {
        return comments.filter(c => c.parent_comment_id === commentId)
    }

    return (
        <div ref={reportRef} className="bg-white p-12 text-black text-sm leading-relaxed max-w-[210mm] mx-auto shadow-none border border-slate-100">
            {/* 頁首資訊 */}
            <div className="flex justify-between mb-8">
                <div>文件編號：AD-04-01-04C</div>
                <div>頁次/總頁數：1 of 2</div>
            </div>

            <h1 className="text-center text-2xl font-bold mb-8">審查意見回覆表</h1>

            {/* 計畫基本資訊表格 */}
            <table className="w-full border-collapse border border-black mb-8">
                <tbody>
                    <tr>
                        <td className="border border-black p-2 font-bold w-32 bg-slate-50">申請編號</td>
                        <td className="border border-black p-2">{protocol?.iacuc_no || basic?.apply_study_number || '尚未指派'}</td>
                    </tr>
                    <tr>
                        <td className="border border-black p-2 font-bold bg-slate-50">研究名稱</td>
                        <td className="border border-black p-2">{protocol?.title}</td>
                    </tr>
                    <tr>
                        <td className="border border-black p-2 font-bold bg-slate-50">計畫主持人</td>
                        <td className="border border-black p-2">{basic?.pi?.name || '未指定'}</td>
                    </tr>
                </tbody>
            </table>

            {/* 1. 執行秘書 */}
            <section className="mb-8">
                <h2 className="font-bold border-b-2 border-black mb-2">初審紀錄</h2>
                <h3 className="font-bold mb-2">1. 執行秘書</h3>
                <table className="w-full border-collapse border border-black">
                    <thead>
                        <tr className="bg-slate-50 font-bold">
                            <td className="border border-black p-2 w-16">項次</td>
                            <td className="border border-black p-2 w-1/2">審查意見</td>
                            <td className="border border-black p-2">申請人回覆</td>
                        </tr>
                    </thead>
                    <tbody>
                        {comments.filter(c => !c.parent_comment_id && c.review_stage === 'PRE_REVIEW').map((c, i) => (
                            <tr key={c.id}>
                                <td className="border border-black p-2">4.1.{i + 1}</td>
                                <td className="border border-black p-2">{c.content}</td>
                                <td className="border border-black p-2">
                                    {getReplies(c.id).map(r => r.content).join('\n')}
                                </td>
                            </tr>
                        ))}
                        {/* 靜態範例顯示，若無資料則顯示空行 */}
                        {comments.filter(c => !c.parent_comment_id && c.review_stage === 'PRE_REVIEW').length === 0 && (
                            <tr>
                                <td className="border border-black p-2 h-12"></td>
                                <td className="border border-black p-2 italic text-slate-400">無初審意見</td>
                                <td className="border border-black p-2"></td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </section>

            {/* 2. 獸醫師 */}
            <section className="mb-8 page-break-inside-avoid">
                <h3 className="font-bold mb-2">2. 獸醫師</h3>
                <table className="w-full border-collapse border border-black">
                    <thead>
                        <tr className="bg-slate-50 font-bold text-center">
                            <td className="border border-black p-2">審查項目</td>
                            <td className="border border-black p-2 w-24">符合 ( v )<br />不符合 ( x )<br />不適用 ( - )</td>
                            <td className="border border-black p-2">審查意見<br />(需補充說明之事項)</td>
                            <td className="border border-black p-2">申請人回覆</td>
                        </tr>
                    </thead>
                    <tbody className="text-center">
                        {(vet_review?.review_form?.items || [
                            "計畫基本資料", "簡述研究目的", "說明動物實驗必要性", "動物實驗試驗設計",
                            "實驗預期結束時機及人道終點", "實驗結束動物處置方式", "有無進行危害性物質實驗",
                            "動物麻醉用藥及方法合理性", "手術操作及術中動物觀察", "手術後照護及術後給藥方式",
                            "實驗動物資料", "動物實驗相關人員資料"
                        ].map(item => ({ item_name: item, compliance: '', comment: '' } as VetReviewItem))).map((item: VetReviewItem, i: number) => (
                            <tr key={i}>
                                <td className="border border-black p-2 text-left">{item.item_name}</td>
                                <td className="border border-black p-2">{item.compliance || ''}</td>
                                <td className="border border-black p-2">{item.comment || ''}</td>
                                <td className="border border-black p-2">
                                    {/* 比照評論回覆邏輯 */}
                                    {item.pi_reply || ''}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                    <tfoot>
                        <tr>
                            <td colSpan={4} className="border border-black p-2">
                                <div className="flex justify-between">
                                    <span>獸醫師簽名：{vet_review?.review_form?.vet_signature ? '(已簽章)' : '____________'}</span>
                                    <span>日期：{vet_review?.review_form?.signed_at ? formatDate(vet_review.review_form.signed_at) : new Date().toLocaleDateString('zh-TW')}</span>
                                </div>
                            </td>
                        </tr>
                    </tfoot>
                </table>
            </section>

            {/* 委員審查紀錄 */}
            <section>
                <h2 className="font-bold border-b-2 border-black mb-4">委員審查紀錄</h2>
                {/* 依委員分組 */}
                {Array.from(new Set(comments.filter(c => c.review_stage === 'UNDER_REVIEW' && !c.parent_comment_id).map(c => c.reviewer_id))).map((reviewerId, idx) => {
                    const reviewerComments = comments.filter(c => c.reviewer_id === reviewerId && !c.parent_comment_id)
                    return (
                        <div key={reviewerId} className="mb-8">
                            <h3 className="font-bold mb-2">{idx + 1}. 委員 {idx + 1}</h3>
                            <table className="w-full border-collapse border border-black">
                                <thead>
                                    <tr className="bg-slate-50 font-bold text-center">
                                        <td className="border border-black p-2 w-16">項次</td>
                                        <td className="border border-black p-2 w-1/4">一審意見</td>
                                        <td className="border border-black p-2 w-1/4">意見回覆</td>
                                        <td className="border border-black p-2 w-1/4">二審意見</td>
                                        <td className="border border-black p-2 w-1/4">意見回覆</td>
                                    </tr>
                                </thead>
                                <tbody>
                                    {reviewerComments.map((c, i) => (
                                        <tr key={c.id}>
                                            <td className="border border-black p-2 text-center">{i + 1}</td>
                                            <td className="border border-black p-2">{c.content}</td>
                                            <td className="border border-black p-2">{getReplies(c.id).map(r => r.content).join('\n')}</td>
                                            <td className="border border-black p-2"></td>
                                            <td className="border border-black p-2"></td>
                                        </tr>
                                    ))}
                                    {reviewerComments.length === 0 && (
                                        <tr className="h-12">
                                            <td className="border border-black p-2"></td>
                                            <td className="border border-black p-2 italic text-slate-400">無審查意見</td>
                                            <td className="border border-black p-2"></td>
                                            <td className="border border-black p-2"></td>
                                            <td className="border border-black p-2"></td>
                                        </tr>
                                    )}
                                </tbody>
                            </table>
                        </div>
                    )
                })}
            </section>

            <footer className="mt-12 text-center text-xs text-slate-500 italic">
                版權為豬博士動物科技股份有限公司所有，禁止任何未經授權的使用<br />
                All Rights Reserved © DrPIG. Unauthorized use in any form is prohibited.
            </footer>

            <style>{`
        @media print {
          .page-break-inside-avoid {
            page-break-inside: avoid;
          }
        }
      `}</style>
        </div>
    )
}
