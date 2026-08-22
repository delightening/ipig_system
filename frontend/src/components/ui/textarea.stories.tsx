import type { Meta, StoryObj } from '@storybook/react-vite'
import { Textarea } from './input'

const meta = {
  title: 'UI/Textarea',
  component: Textarea,
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
  argTypes: {
    placeholder: { control: 'text' },
    disabled: { control: 'boolean' },
    error: { control: 'boolean' },
    rows: { control: 'number' },
  },
} satisfies Meta<typeof Textarea>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    placeholder: '請輸入內容...',
    rows: 4,
  },
  decorators: [(Story) => <div className="w-80"><Story /></div>],
}

export const WithContent: Story = {
  name: '含文字內容',
  args: {
    defaultValue: '觀察紀錄：動物狀態正常，進食量良好，無異常行為。',
    rows: 4,
  },
  decorators: [(Story) => <div className="w-80"><Story /></div>],
}

export const WithError: Story = {
  name: '錯誤狀態',
  args: {
    placeholder: '此欄位必填',
    error: true,
    rows: 3,
  },
  decorators: [(Story) => <div className="w-80"><Story /></div>],
}

export const Disabled: Story = {
  name: '停用狀態',
  args: {
    defaultValue: '此欄位已被鎖定，無法編輯。',
    disabled: true,
    rows: 3,
  },
  decorators: [(Story) => <div className="w-80"><Story /></div>],
}
