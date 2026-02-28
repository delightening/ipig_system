import type { Meta, StoryObj } from '@storybook/react-vite'
import { Badge } from './badge'

const meta = {
  title: 'UI/Badge',
  component: Badge,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'secondary', 'destructive', 'outline', 'success', 'warning'],
    },
  },
} satisfies Meta<typeof Badge>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    children: '標籤',
  },
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-2">
      <Badge variant="default">預設</Badge>
      <Badge variant="secondary">次要</Badge>
      <Badge variant="destructive">危險</Badge>
      <Badge variant="outline">外框</Badge>
      <Badge variant="success">成功</Badge>
      <Badge variant="warning">警告</Badge>
    </div>
  ),
}

export const UseCases: Story = {
  name: '應用場景',
  render: () => (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">訂單狀態：</span>
        <Badge variant="success">已完成</Badge>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">庫存狀態：</span>
        <Badge variant="warning">低庫存</Badge>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-sm font-medium">審核狀態：</span>
        <Badge variant="destructive">已退回</Badge>
      </div>
    </div>
  ),
}
