import type { Meta, StoryObj } from '@storybook/react-vite'
import { Switch } from './switch'
import { Label } from './label'

const meta = {
  title: 'UI/Switch',
  component: Switch,
  tags: ['autodocs'],
  argTypes: {
    disabled: { control: 'boolean' },
    defaultChecked: { control: 'boolean' },
  },
} satisfies Meta<typeof Switch>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithLabel: Story = {
  render: () => (
    <div className="flex items-center gap-2">
      <Switch id="dark-mode" />
      <Label htmlFor="dark-mode">深色模式</Label>
    </div>
  ),
}

export const Checked: Story = {
  args: {
    defaultChecked: true,
  },
}

export const Disabled: Story = {
  args: {
    disabled: true,
  },
}

export const SettingsExample: Story = {
  name: '設定頁面範例',
  render: () => (
    <div className="w-[320px] space-y-4">
      <div className="flex items-center justify-between">
        <Label>通知推播</Label>
        <Switch defaultChecked />
      </div>
      <div className="flex items-center justify-between">
        <Label>自動儲存</Label>
        <Switch defaultChecked />
      </div>
      <div className="flex items-center justify-between">
        <Label>深色模式</Label>
        <Switch />
      </div>
      <div className="flex items-center justify-between">
        <Label className="text-muted-foreground">進階功能（即將推出）</Label>
        <Switch disabled />
      </div>
    </div>
  ),
}
