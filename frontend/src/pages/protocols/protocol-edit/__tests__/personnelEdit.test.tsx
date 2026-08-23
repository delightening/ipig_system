import { render, screen, fireEvent } from '@testing-library/react'
import { vi } from 'vitest'
import i18n from '@/lib/i18n'
import { AddPersonnelDialog, resolveStaffPosition } from '../AddPersonnelDialog'
import { SectionPersonnel } from '../SectionPersonnel'
import { defaultFormData } from '../constants'
import type { ProtocolPerson } from '@/types/protocol'

// jsdom navigator 預設 en-US，強制 zh-TW 讓字串判斷穩定；
// 結束後還原語系，避免污染共用 i18n singleton 影響其他測試
let previousLanguage: string

beforeAll(async () => {
  previousLanguage = i18n.language
  await i18n.changeLanguage('zh-TW')
})

afterAll(async () => {
  await i18n.changeLanguage(previousLanguage)
})

const samplePerson: ProtocolPerson = {
  id: 9,
  name: '王小明',
  position: '',
  roles: ['b'],
  roles_other_text: '',
  years_experience: 3,
  trainings: ['A'],
  trainings_other_text: '',
  training_certificates: [{ training_code: 'A', certificate_no: 'CERT-1' }],
}

describe('AddPersonnelDialog 編輯模式', () => {
  it('編輯模式預填既有資料，存檔呼叫 onSave(index, data) 而非 onAdd（修正而非刪除重建）', () => {
    const onAdd = vi.fn()
    const onSave = vi.fn()
    render(
      <AddPersonnelDialog
        open
        onOpenChange={() => {}}
        staffMembers={[]}
        isIACUCStaff={false}
        onAdd={onAdd}
        editingIndex={2}
        editingPerson={samplePerson}
        onSave={onSave}
      />,
    )

    const nameInput = screen.getByDisplayValue('王小明')
    expect(nameInput).toBeInTheDocument()

    fireEvent.change(nameInput, { target: { value: '王大明' } })
    fireEvent.click(screen.getByRole('button', { name: /儲存/ }))

    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith(2, expect.objectContaining({ name: '王大明' }))
    expect(onAdd).not.toHaveBeenCalled()
  })

  it('編輯模式缺 onSave 時不 fallback 到 onAdd（避免重複列）', () => {
    const onAdd = vi.fn()
    render(
      <AddPersonnelDialog
        open
        onOpenChange={() => {}}
        staffMembers={[]}
        isIACUCStaff={false}
        onAdd={onAdd}
        editingIndex={1}
        editingPerson={samplePerson}
      />,
    )

    fireEvent.click(screen.getByRole('button', { name: /儲存/ }))
    expect(onAdd).not.toHaveBeenCalled()
  })

  it('新增模式維持原本行為（按鈕為「確認新增」、無「儲存」）', () => {
    render(
      <AddPersonnelDialog
        open
        onOpenChange={() => {}}
        staffMembers={[]}
        isIACUCStaff={false}
        onAdd={() => {}}
      />,
    )

    expect(screen.getByRole('button', { name: /確認新增/ })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^儲存$/ })).not.toBeInTheDocument()
  })
})

describe('resolveStaffPosition：職稱來自人事主檔', () => {
  it('有設定就用 users.position，不再硬編固定職稱', () => {
    expect(resolveStaffPosition({ position: '研究助理' })).toBe('研究助理')
    expect(resolveStaffPosition({ position: '實習生' })).toBe('實習生')
  })

  it('未設定時留空，不在輸入階段塞 §8 預設值', () => {
    // §8 的空值預設是顯示層的職責（SectionPersonnel / pdf_export），
    // 在這裡塞值會讓「沒設定」與「剛好等於預設值」分不出來。
    expect(resolveStaffPosition({})).toBe('')
    expect(resolveStaffPosition({ position: undefined })).toBe('')
  })

  it('只有空白的職稱視同未設定', () => {
    expect(resolveStaffPosition({ position: '   ' })).toBe('')
    expect(resolveStaffPosition({ position: ' 獸醫師 ' })).toBe('獸醫師')
  })
})

describe('SectionPersonnel 編輯入口', () => {
  it('每位人員列提供編輯按鈕，點擊呼叫 onEditPersonnel(index)', () => {
    const onEditPersonnel = vi.fn()
    render(
      <SectionPersonnel
        formData={defaultFormData}
        updateWorkingContent={() => {}}
        setFormData={() => {}}
        t={i18n.t}
        isIACUCStaff={false}
        formVersion="F"
        onAddPersonnel={() => {}}
        onEditPersonnel={onEditPersonnel}
      />,
    )

    const editButtons = screen.getAllByRole('button', { name: '編輯' })
    expect(editButtons.length).toBe(defaultFormData.working_content.personnel.length)

    fireEvent.click(editButtons[0])
    expect(onEditPersonnel).toHaveBeenCalledWith(0)
  })
})
