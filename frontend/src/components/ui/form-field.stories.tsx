import type { Meta, StoryObj } from '@storybook/react-vite'
import { FormField } from './form-field'
import { Input, Textarea } from './input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './select'

const meta = {
  title: 'UI/FormField',
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <div className="w-80">
      <FormField label="動物名稱" htmlFor="name">
        <Input id="name" placeholder="輸入名稱" />
      </FormField>
    </div>
  ),
}

export const Required: Story = {
  name: '必填欄位',
  render: () => (
    <div className="w-80">
      <FormField label="Email" htmlFor="email" required>
        <Input id="email" type="email" placeholder="user@example.com" />
      </FormField>
    </div>
  ),
}

export const WithError: Story = {
  name: '含錯誤訊息',
  render: () => (
    <div className="w-80">
      <FormField label="密碼" htmlFor="password" required error="密碼長度至少 8 個字元">
        <Input id="password" type="password" error />
      </FormField>
    </div>
  ),
}

export const WithTextarea: Story = {
  name: '搭配 Textarea',
  render: () => (
    <div className="w-80">
      <FormField label="備註" htmlFor="notes">
        <Textarea id="notes" placeholder="輸入備註..." rows={4} />
      </FormField>
    </div>
  ),
}

export const WithSelect: Story = {
  name: '搭配 Select',
  render: () => (
    <div className="w-80">
      <FormField label="品種" htmlFor="species" required>
        <Select>
          <SelectTrigger>
            <SelectValue placeholder="選擇品種" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="landrace">藍瑞斯</SelectItem>
            <SelectItem value="yorkshire">約克夏</SelectItem>
            <SelectItem value="duroc">杜洛克</SelectItem>
          </SelectContent>
        </Select>
      </FormField>
    </div>
  ),
}

export const FormLayout: Story = {
  name: '表單佈局組合',
  render: () => (
    <div className="w-96 space-y-4 rounded-lg border p-6">
      <h3 className="text-lg font-semibold">新增動物</h3>
      <FormField label="耳號" htmlFor="ear_tag" required>
        <Input id="ear_tag" placeholder="SPF-001" />
      </FormField>
      <FormField label="品種" required>
        <Select>
          <SelectTrigger>
            <SelectValue placeholder="選擇品種" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="landrace">藍瑞斯</SelectItem>
            <SelectItem value="yorkshire">約克夏</SelectItem>
          </SelectContent>
        </Select>
      </FormField>
      <FormField label="體重" htmlFor="weight" required error="請輸入體重">
        <Input id="weight" type="number" placeholder="kg" error />
      </FormField>
      <FormField label="備註" htmlFor="notes">
        <Textarea id="notes" placeholder="其他資訊..." rows={3} />
      </FormField>
    </div>
  ),
}
