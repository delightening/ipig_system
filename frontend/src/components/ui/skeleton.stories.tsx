import type { Meta, StoryObj } from '@storybook/react-vite'
import { Skeleton, InlineSkeleton } from './skeleton'

const meta = {
  title: 'UI/Skeleton',
  component: Skeleton,
  tags: ['autodocs'],
  parameters: {
    layout: 'padded',
  },
} satisfies Meta<typeof Skeleton>

export default meta
type Story = StoryObj<typeof meta>

export const Table: Story = {
  name: '表格骨架屏',
  args: {
    variant: 'table',
    rows: 5,
    columns: 4,
  },
}

export const Cards: Story = {
  name: '卡片骨架屏',
  args: {
    variant: 'card',
    count: 3,
  },
}

export const Form: Story = {
  name: '表單骨架屏',
  args: {
    variant: 'form',
    fields: 4,
  },
}

export const InlineSkeletonStory: StoryObj = {
  name: '行內骨架',
  render: () => (
    <p className="text-sm">
      目前共有 <InlineSkeleton className="w-12" /> 筆資料
    </p>
  ),
}
