import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { Slider } from './slider'

const meta = {
  title: 'UI/Slider',
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: function Render() {
    const [value, setValue] = useState(100)
    return (
      <div className="w-96">
        <Slider value={value} onChange={setValue} />
      </div>
    )
  },
}

export const WithLabel: Story = {
  name: '含標籤與單位',
  render: function Render() {
    const [value, setValue] = useState(30)
    return (
      <div className="w-96">
        <Slider
          value={value}
          onChange={setValue}
          min={1}
          max={90}
          label="效期預警天數"
          unit="天"
          quickValues={[7, 14, 30, 60]}
        />
      </div>
    )
  },
}

export const CustomRange: Story = {
  name: '自訂範圍',
  render: function Render() {
    const [value, setValue] = useState(25)
    return (
      <div className="w-96">
        <Slider
          value={value}
          onChange={setValue}
          min={0}
          max={100}
          step={5}
          label="溫度設定"
          unit="°C"
          quickValues={[18, 22, 25, 28]}
        />
      </div>
    )
  },
}

export const WithUnitOptions: Story = {
  name: '含單位切換',
  render: function Render() {
    const [value, setValue] = useState(500)
    const [unit, setUnit] = useState('g')
    return (
      <div className="w-96">
        <Slider
          value={value}
          onChange={setValue}
          min={0}
          max={5000}
          step={10}
          label="重量"
          quickValues={[100, 500, 1000, 2000]}
          unitOptions={[
            { value: 'g', label: '公克' },
            { value: 'kg', label: '公斤' },
          ]}
          selectedUnit={unit}
          onUnitChange={setUnit}
        />
      </div>
    )
  },
}

export const Disabled: Story = {
  name: '停用狀態',
  render: function Render() {
    const [value] = useState(50)
    return (
      <div className="w-96">
        <Slider
          value={value}
          onChange={() => {}}
          label="已鎖定的設定"
          unit="天"
          disabled
        />
      </div>
    )
  },
}
