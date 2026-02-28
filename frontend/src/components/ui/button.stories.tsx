import type { Meta, StoryObj } from '@storybook/react-vite'
import { Button } from './button'
import { Loader2, Mail, Plus, Trash2 } from 'lucide-react'

const meta = {
  title: 'UI/Button',
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'destructive', 'outline', 'secondary', 'ghost', 'link'],
    },
    size: {
      control: 'select',
      options: ['default', 'sm', 'lg', 'icon'],
    },
    disabled: { control: 'boolean' },
  },
} satisfies Meta<typeof Button>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    children: '按鈕',
  },
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      <Button variant="default">Primary</Button>
      <Button variant="secondary">Secondary</Button>
      <Button variant="destructive">Destructive</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="link">Link</Button>
    </div>
  ),
}

export const AllSizes: Story = {
  render: () => (
    <div className="flex items-center gap-3">
      <Button size="sm">小</Button>
      <Button size="default">預設</Button>
      <Button size="lg">大</Button>
      <Button size="icon"><Plus className="h-4 w-4" /></Button>
    </div>
  ),
}

export const WithIcon: Story = {
  render: () => (
    <div className="flex items-center gap-3">
      <Button><Mail className="mr-2 h-4 w-4" />寄送郵件</Button>
      <Button variant="destructive"><Trash2 className="mr-2 h-4 w-4" />刪除</Button>
    </div>
  ),
}

export const Loading: Story = {
  render: () => (
    <Button disabled>
      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
      處理中...
    </Button>
  ),
}

export const Disabled: Story = {
  args: {
    children: '已停用',
    disabled: true,
  },
}
