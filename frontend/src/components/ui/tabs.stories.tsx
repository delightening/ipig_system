import type { Meta, StoryObj } from '@storybook/react-vite'
import { Tabs, TabsContent, TabsList, TabsTrigger } from './tabs'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from './card'
import { Input } from './input'
import { Label } from './label'
import { Button } from './button'

const meta = {
  title: 'UI/Tabs',
  tags: ['autodocs'],
  parameters: { layout: 'centered' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Tabs defaultValue="overview" className="w-[500px]">
      <TabsList>
        <TabsTrigger value="overview">概覽</TabsTrigger>
        <TabsTrigger value="details">詳情</TabsTrigger>
        <TabsTrigger value="history">歷史紀錄</TabsTrigger>
      </TabsList>
      <TabsContent value="overview" className="p-4">
        <p className="text-sm text-muted-foreground">這是概覽頁籤的內容。</p>
      </TabsContent>
      <TabsContent value="details" className="p-4">
        <p className="text-sm text-muted-foreground">這是詳情頁籤的內容。</p>
      </TabsContent>
      <TabsContent value="history" className="p-4">
        <p className="text-sm text-muted-foreground">這是歷史紀錄頁籤的內容。</p>
      </TabsContent>
    </Tabs>
  ),
}

export const WithCards: Story = {
  name: '含 Card 內容',
  render: () => (
    <Tabs defaultValue="account" className="w-[500px]">
      <TabsList className="grid w-full grid-cols-2">
        <TabsTrigger value="account">帳號設定</TabsTrigger>
        <TabsTrigger value="password">密碼設定</TabsTrigger>
      </TabsList>
      <TabsContent value="account">
        <Card>
          <CardHeader>
            <CardTitle>帳號</CardTitle>
            <CardDescription>修改您的帳號資訊</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="name">姓名</Label>
              <Input id="name" defaultValue="王小明" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="email">Email</Label>
              <Input id="email" defaultValue="admin@ipig.local" />
            </div>
            <Button>儲存變更</Button>
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="password">
        <Card>
          <CardHeader>
            <CardTitle>密碼</CardTitle>
            <CardDescription>修改您的登入密碼</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="current">目前密碼</Label>
              <Input id="current" type="password" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new">新密碼</Label>
              <Input id="new" type="password" />
            </div>
            <Button>更新密碼</Button>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  ),
}

export const ManyTabs: Story = {
  name: '多頁籤（自動換行）',
  render: () => (
    <Tabs defaultValue="info" className="w-[500px]">
      <TabsList>
        <TabsTrigger value="info">基本資訊</TabsTrigger>
        <TabsTrigger value="observation">觀察紀錄</TabsTrigger>
        <TabsTrigger value="surgery">手術紀錄</TabsTrigger>
        <TabsTrigger value="weight">體重紀錄</TabsTrigger>
        <TabsTrigger value="vaccine">疫苗紀錄</TabsTrigger>
        <TabsTrigger value="sacrifice">犧牲紀錄</TabsTrigger>
      </TabsList>
      <TabsContent value="info" className="p-4">
        <p className="text-sm">動物基本資訊頁籤</p>
      </TabsContent>
    </Tabs>
  ),
}
