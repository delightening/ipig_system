import type { Meta, StoryObj } from '@storybook/react-vite'
import { Input, Textarea } from './input'
import { Label } from './label'
import { Search } from 'lucide-react'

const meta = {
  title: 'UI/Input',
  component: Input,
  tags: ['autodocs'],
  argTypes: {
    type: {
      control: 'select',
      options: ['text', 'email', 'password', 'number', 'search', 'tel', 'url'],
    },
    disabled: { control: 'boolean' },
    error: { control: 'boolean' },
    placeholder: { control: 'text' },
  },
} satisfies Meta<typeof Input>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    placeholder: '請輸入...',
  },
}

export const WithLabel: Story = {
  render: () => (
    <div className="grid w-full max-w-sm gap-1.5">
      <Label htmlFor="email">Email</Label>
      <Input type="email" id="email" placeholder="user@example.com" />
    </div>
  ),
}

export const WithError: Story = {
  render: () => (
    <div className="grid w-full max-w-sm gap-1.5">
      <Label htmlFor="name">名稱</Label>
      <Input id="name" error placeholder="此欄位必填" />
      <p className="text-sm text-red-500">此欄位為必填</p>
    </div>
  ),
}

export const Disabled: Story = {
  args: {
    placeholder: '已停用',
    disabled: true,
  },
}

export const WithIcon: Story = {
  render: () => (
    <div className="relative w-full max-w-sm">
      <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
      <Input className="pl-9" placeholder="搜尋..." />
    </div>
  ),
}

export const TextareaStory: StoryObj<typeof Textarea> = {
  render: () => (
    <div className="grid w-full max-w-sm gap-1.5">
      <Label htmlFor="message">備註</Label>
      <Textarea id="message" placeholder="請輸入備註內容..." />
    </div>
  ),
  name: 'Textarea',
}
