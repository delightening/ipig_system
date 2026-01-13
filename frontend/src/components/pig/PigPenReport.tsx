import React from 'react'
import { PigListItem } from '@/lib/api'

interface PigListItemExtended extends PigListItem { }

interface PigPenReportProps {
    data: { pen_location: string; pigs: PigListItemExtended[] }[]
    onClose: () => void
}

export const PigPenReport: React.FC<PigPenReportProps> = ({ data, onClose }) => {
    const [docId, setDocId] = React.useState('AD-05-01-02C')
    const pigsByPen = new Map(data.map(item => [item.pen_location, item.pigs]))

    const getStatusCircle = (pigs: PigListItemExtended[]) => {
        const isAssigned = pigs.some(p => p.status === 'assigned' || p.status === 'in_experiment')
        return (
            <div className={`w-3.5 h-3.5 rounded-full border border-black ${pigs.length === 0 ? '' : isAssigned ? 'bg-black' : 'bg-white'}`} />
        )
    }

    const PenUnit = ({ id, color = 'bg-white', reverse = false, small = false, noBorder = false }: { id: string; color?: string; reverse?: boolean; small?: boolean; noBorder?: boolean }) => {
        const pigs = pigsByPen.get(id) || []
        const earTags = pigs.map(p => p.ear_tag).join('.')
        const circle = <div className={`${small ? 'w-4' : 'w-5'} flex justify-center items-center h-full`}>{getStatusCircle(pigs)}</div>
        const tag = (
            <div className={`flex-1 ${noBorder ? '' : 'border border-black'} h-full flex items-center justify-center font-bold overflow-hidden px-0.5 ${color} ${small ? 'text-[8px]' : 'text-[9px]'}`}>
                {earTags}
            </div>
        )
        return <div className="flex items-center w-full h-full">{reverse ? <>{circle}{tag}</> : <>{tag}{circle}</>}</div>
    }

    const PenUnitF = ({ id }: { id: string }) => {
        const pigs = pigsByPen.get(id) || []
        return (
            <div className="relative w-full h-full">
                <div className="w-full h-full border border-black bg-[#FFCC00] flex items-center justify-center font-bold text-[8px]">{pigs.map(p => p.ear_tag).join('.')}</div>
                <div className="absolute left-1/2 -bottom-[6px] -translate-x-1/2 z-10">{getStatusCircle(pigs)}</div>
            </div>
        )
    }

    return (
        <div className="fixed inset-0 bg-white z-[9999] overflow-auto p-8 print:p-0 report-font" id="print-area">
            <div className="w-[210mm] mx-auto bg-white p-6 print:p-4">

                {/* Header Info */}
                <div className="flex justify-between items-center text-[10px] mb-2">
                    <div className="flex items-center gap-1">文件編號 <input value={docId} onChange={e => setDocId(e.target.value)} className="border-none p-0 focus:ring-0 w-32 font-medium print:bg-transparent" /></div>
                    <div className="font-bold text-[12px]">豬博士動物科技股份有限公司</div>
                    <div>頁次/總頁數 1 of 1</div>
                </div>

                {/* Inspector & Date */}
                <div className="flex flex-col items-center mb-2 text-sm">
                    <div className="flex items-center gap-2">
                        <span className="font-bold text-base">巡視人：</span><div className="w-56 border-b border-black h-7" />
                        <div className="flex items-center gap-4 ml-6 border border-black rounded-full px-4 py-0.5">
                            <div className="flex items-center gap-1.5"><div className="w-4 h-4 rounded-full border border-black" /><span className="text-xs font-bold">AM</span></div>
                            <div className="flex items-center gap-1.5"><div className="w-4 h-4 rounded-full border border-black" /><span className="text-xs font-bold">PM</span></div>
                        </div>
                    </div>
                    <div className="flex items-center gap-2 mt-1">
                        <span className="font-bold text-base">巡視日期：</span><div className="w-56 border-b border-black h-7" />
                        <div className="w-4 h-4 rounded-full border border-black ml-4" />
                    </div>
                </div>

                {/* Main Grid */}
                <div className="flex border-t-[3px] border-black pt-2">
                    {/* Building A */}
                    <div className="w-1/2 pr-4 flex flex-col">
                        <div className="flex flex-col">
                            {[18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1].map(n => (
                                <div key={`d-${n}`} className="h-6 flex">
                                    <div className="flex-1"><PenUnit id={`D${String(n).padStart(2, '0')}`} color="bg-[#FDD9C4]" /></div>
                                    <div className="flex-1">{n <= 15 ? <PenUnit id={`D${String(n + 18).padStart(2, '0')}`} color="bg-[#FDD9C4]" reverse /> : <div className="border-l border-black h-full" />}</div>
                                </div>
                            ))}
                            <div className="h-6 flex items-center justify-center font-bold text-lg">D</div>
                        </div>
                        <div className="flex flex-col">
                            {[10, 9, 8, 7, 6, 5, 4, 3, 2, 1].map(n => (
                                <div key={`c-${n}`} className="h-6 flex">
                                    <div className="flex-1"><PenUnit id={`C${String(n).padStart(2, '0')}`} color="bg-[#B5E1F4]" /></div>
                                    <div className="flex-1"><PenUnit id={`C${String(n + 10).padStart(2, '0')}`} color="bg-[#B5E1F4]" reverse /></div>
                                </div>
                            ))}
                            <div className="h-6 flex items-center justify-center font-bold text-lg">C</div>
                        </div>
                    </div>

                    {/* Building B */}
                    <div className="w-1/2 pl-4 flex flex-col">
                        <div className="grid grid-cols-2" style={{ gridTemplateRows: 'repeat(18, 1.5rem)' }}>
                            {Array.from({ length: 15 }).map((_, i) => (
                                <div key={`e-t-${i}`} style={{ gridRow: i + 4 }}><PenUnit id={`E${String(25 - i).padStart(2, '0')}`} color="bg-[#C070C0]" /></div>
                            ))}
                            {[{ id: 'G06', r: 1, s: 3 }, { id: 'G05', r: 4, s: 3 }, { id: 'G04', r: 7, s: 3 }, { id: 'G03', r: 10, s: 3 }, { id: 'G02', r: 13, s: 3 }, { id: 'G01', r: 16, s: 3 }].map(g => (
                                <div key={g.id} style={{ gridColumn: 2, gridRow: `${g.r} / span ${g.s}` }} className="border-l border-black"><PenUnit id={g.id} color="bg-[#34C759]" reverse /></div>
                            ))}
                        </div>
                        <div className="h-6 flex"><div className="flex-1 flex items-center justify-center font-bold text-lg text-[#C070C0]">E</div><div className="flex-1 flex items-center justify-center font-bold text-lg text-[#34C759]">G</div></div>
                        <div className="grid grid-cols-2" style={{ gridTemplateRows: 'repeat(10, 1.5rem)' }}>
                            {[10, 9, 8, 7, 6, 5, 4, 3, 2, 1].map((n, idx) => (
                                <div key={`e-b-${n}`} style={{ gridRow: idx + 1 }}><PenUnit id={`E${String(n).padStart(2, '0')}`} color="bg-[#C070C0]" /></div>
                            ))}
                            {[{ r: 1, ids: ['F04', 'F05', 'F06'] }, { r: 4, ids: ['F01', 'F02', 'F03'] }].map(f => (
                                <div key={f.ids[0]} style={{ gridColumn: 2, gridRow: `${f.r} / span 2` }} className="flex">
                                    <div className="w-5" />{f.ids.map((id, i) => <div key={id} className={`flex-1 ${i > 0 ? '-ml-[1px]' : ''}`}><PenUnitF id={id} /></div>)}
                                </div>
                            ))}
                        </div>
                        <div className="h-6 flex"><div className="flex-1" /><div className="flex-1 flex items-center justify-center font-bold text-lg text-[#FFCC00] mt-[-8px]">F</div></div>
                    </div>
                </div>

                {/* Bottom Zones */}
                <div className="flex mt-2">
                    <div className="w-1/2 pr-4">{[10, 9, 8, 7, 6, 5, 4, 3, 2, 1].map(n => (<div key={`a-${n}`} className="h-6 flex"><PenUnit id={`A${String(n).padStart(2, '0')}`} color="bg-[#FFFF00]" /><PenUnit id={`A${String(n + 10).padStart(2, '0')}`} color="bg-[#FFFF00]" reverse /></div>))}</div>
                    <div className="w-1/2 pl-4">{[10, 9, 8, 7, 6, 5, 4, 3, 2, 1].map(n => (<div key={`b-${n}`} className="h-6 flex"><PenUnit id={`B${String(n).padStart(2, '0')}`} color="bg-[#FF0000]" /><PenUnit id={`B${String(n + 10).padStart(2, '0')}`} color="bg-[#FF0000]" reverse /></div>))}</div>
                </div>
                <div className="flex h-6 font-bold text-lg"><div className="flex-1 text-center">A</div><div className="flex-1 text-center">B</div></div>

                {/* Footer */}
                <div className="mt-4 pt-4 border-t-2 border-black text-xs space-y-2">
                    <div className="flex items-center gap-2"><div className="w-4 h-4 border border-black" />全場豬隻精神、食慾、活動力、外觀正常。</div>
                    <div className="flex items-start gap-2"><div className="w-4 h-4 border border-black mt-0.5 shrink-0" /><div>豬隻異常，耳號________________，請填寫「TU-03-04-01試驗豬隻異常狀況紀錄表」並依照「TU-03-04-00試驗豬隻異常之處理標準作業程序書」規範處理。</div></div>
                    <div className="text-[9px] text-center pt-2 uppercase font-sans">版權為豬博士動物科技股份有限公司所有，禁止任何未經授權的使用<br />All Rights Reserved © DrPIG. Unauthorized use in any form is prohibited.</div>
                </div>

                {/* Controls */}
                <div className="mt-8 flex justify-center gap-4 print:hidden">
                    <button onClick={onClose} className="px-6 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 font-bold">關閉</button>
                    <button onClick={() => window.print()} className="px-8 py-2 bg-black text-white rounded-lg hover:opacity-80 font-bold">列印表單</button>
                </div>
            </div>

            <style>{`
        .report-font { font-family: "Times New Roman", "標楷體", "DFKai-SB", "BiauKai", serif; }
        @media print {
          body * { visibility: hidden; }
          #print-area, #print-area * { visibility: visible; }
          #print-area { position: absolute; left: 0; top: 0; width: 100%; margin: 0; padding: 0; }
          @page { size: A4 portrait; margin: 0; }
        }
      `}</style>
        </div>
    )
}
