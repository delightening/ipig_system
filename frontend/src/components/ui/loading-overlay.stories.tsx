import type { Meta, StoryObj } from '@storybook/react-vite'
import { LoadingOverlay } from './loading-overlay'

const meta = {
  title: 'UI/LoadingOverlay',
  component: LoadingOverlay,
  tags: ['autodocs'],
  argTypes: {
    message: { control: 'text' },
    fullScreen: { control: 'boolean' },
  },
} satisfies Meta<typeof LoadingOverlay>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    message: '載入中...',
  },
  decorators: [
    (Story) => (
      <div className="w-80 h-60 border rounded-lg relative">
        <Story />
      </div>
    ),
  ],
}

export const CustomMessage: Story = {
  name: '自訂訊息',
  args: {
    message: '正在載入動物清單...',
  },
  decorators: [
    (Story) => (
      <div className="w-80 h-60 border rounded-lg relative">
        <Story />
      </div>
    ),
  ],
}

export const NoMessage: Story = {
  name: '無訊息',
  args: {
    message: '',
  },
  decorators: [
    (Story) => (
      <div className="w-80 h-40 border rounded-lg relative">
        <Story />
      </div>
    ),
  ],
}

export const InCardContext: Story = {
  name: '在 Card 內使用',
  render: () => (
    <div className="w-96 rounded-lg border bg-white p-6 space-y-3">
      <h3 className="text-lg font-semibold">最近觀察紀錄</h3>
      <LoadingOverlay message="正在載入紀錄..." />
    </div>
  ),
}
