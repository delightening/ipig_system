import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from './select'
import { Label } from './label'

const meta = {
  title: 'UI/Select',
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <div className="w-64">
      <Select>
        <SelectTrigger>
          <SelectValue placeholder="選擇水果" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="apple">蘋果</SelectItem>
          <SelectItem value="banana">香蕉</SelectItem>
          <SelectItem value="grape">葡萄</SelectItem>
          <SelectItem value="orange">柳橙</SelectItem>
        </SelectContent>
      </Select>
    </div>
  ),
}

export const WithGroups: Story = {
  name: '分組選項',
  render: () => (
    <div className="w-64">
      <Select>
        <SelectTrigger>
          <SelectValue placeholder="選擇倉庫" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectLabel>北區</SelectLabel>
            <SelectItem value="taipei">台北倉庫</SelectItem>
            <SelectItem value="taoyuan">桃園倉庫</SelectItem>
          </SelectGroup>
          <SelectSeparator />
          <SelectGroup>
            <SelectLabel>南區</SelectLabel>
            <SelectItem value="tainan">台南倉庫</SelectItem>
            <SelectItem value="kaohsiung">高雄倉庫</SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>
    </div>
  ),
}

export const WithDisabledItems: Story = {
  name: '含停用選項',
  render: () => (
    <div className="w-64">
      <Select>
        <SelectTrigger>
          <SelectValue placeholder="成本計算方式" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="weighted_average">加權平均法</SelectItem>
          <SelectItem value="moving_average">移動平均法</SelectItem>
          <SelectItem value="fifo" disabled>先進先出 (v0.2)</SelectItem>
          <SelectItem value="lifo" disabled>後進先出 (v0.2)</SelectItem>
        </SelectContent>
      </Select>
    </div>
  ),
}

export const Controlled: Story = {
  name: '受控模式',
  render: function Render() {
    const [value, setValue] = useState('draft')
    return (
      <div className="w-64 space-y-2">
        <Label>審核狀態</Label>
        <Select value={value} onValueChange={setValue}>
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="draft">草稿</SelectItem>
            <SelectItem value="submitted">已送審</SelectItem>
            <SelectItem value="approved">已核准</SelectItem>
            <SelectItem value="rejected">已退回</SelectItem>
          </SelectContent>
        </Select>
        <p className="text-xs text-muted-foreground">目前值：{value}</p>
      </div>
    )
  },
}

export const Disabled: Story = {
  name: '停用狀態',
  render: () => (
    <div className="w-64">
      <Select disabled>
        <SelectTrigger>
          <SelectValue placeholder="無法選擇" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="a">選項 A</SelectItem>
        </SelectContent>
      </Select>
    </div>
  ),
}
