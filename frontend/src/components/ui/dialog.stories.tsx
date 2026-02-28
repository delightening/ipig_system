import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './dialog'
import { Button } from './button'
import { Input } from './input'
import { Label } from './label'

const meta = {
  title: 'UI/Dialog',
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">開啟 Dialog</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>基本對話框</DialogTitle>
          <DialogDescription>這是一個基本的 Dialog 元件範例。</DialogDescription>
        </DialogHeader>
        <p className="text-sm text-muted-foreground">
          Dialog 可用於顯示需要使用者注意的重要資訊，或收集使用者輸入。
        </p>
        <DialogFooter>
          <Button variant="outline">取消</Button>
          <Button>確認</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}

export const WithForm: Story = {
  name: '含表單',
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button>新增動物</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>新增動物</DialogTitle>
          <DialogDescription>輸入新動物的基本資訊</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="name" className="text-right">名稱</Label>
            <Input id="name" placeholder="輸入動物名稱" className="col-span-3" />
          </div>
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="species" className="text-right">品種</Label>
            <Input id="species" placeholder="輸入品種" className="col-span-3" />
          </div>
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="weight" className="text-right">體重</Label>
            <Input id="weight" type="number" placeholder="kg" className="col-span-3" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline">取消</Button>
          <Button>儲存</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}

export const Destructive: Story = {
  name: '危險操作',
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="destructive">刪除紀錄</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>確認刪除</DialogTitle>
          <DialogDescription>
            此操作無法復原。您確定要刪除這筆紀錄嗎？
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline">取消</Button>
          <Button variant="destructive">確認刪除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}
