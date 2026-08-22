import type { Meta, StoryObj } from '@storybook/react-vite'
import { Checkbox } from './checkbox'

const meta = {
  title: 'UI/Checkbox',
  component: Checkbox,
  tags: ['autodocs'],
  argTypes: {
    label: { control: 'text' },
    disabled: { control: 'boolean' },
    checked: { control: 'boolean' },
  },
} satisfies Meta<typeof Checkbox>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    label: '同意條款',
  },
}

export const Checked: Story = {
  args: {
    label: '已勾選',
    defaultChecked: true,
  },
}

export const Disabled: Story = {
  args: {
    label: '已停用',
    disabled: true,
  },
}

export const CheckboxGroup: Story = {
  name: '核取方塊群組',
  render: () => (
    <div className="flex flex-col gap-3">
      <Checkbox label="飼料管理" defaultChecked />
      <Checkbox label="疫苗管理" />
      <Checkbox label="藥品管理" defaultChecked />
      <Checkbox label="耗材管理" />
    </div>
  ),
}
