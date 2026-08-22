import type { Meta, StoryObj } from '@storybook/react-vite'
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from './card'
import { Button } from './button'
import { Badge } from './badge'

const meta = {
  title: 'UI/Card',
  component: Card,
  tags: ['autodocs'],
} satisfies Meta<typeof Card>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Card className="w-[380px]">
      <CardHeader>
        <CardTitle>卡片標題</CardTitle>
        <CardDescription>卡片說明文字，用於補充說明。</CardDescription>
      </CardHeader>
      <CardContent>
        <p>這是卡片的主要內容區域。</p>
      </CardContent>
      <CardFooter className="flex justify-between">
        <Button variant="outline">取消</Button>
        <Button>確認</Button>
      </CardFooter>
    </Card>
  ),
}

export const StatsCard: Story = {
  name: '統計卡片',
  render: () => (
    <div className="grid grid-cols-3 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardDescription>本月營收</CardDescription>
          <CardTitle className="text-3xl">$45,231</CardTitle>
        </CardHeader>
        <CardContent>
          <Badge variant="success">+20.1%</Badge>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardDescription>庫存品項</CardDescription>
          <CardTitle className="text-3xl">2,350</CardTitle>
        </CardHeader>
        <CardContent>
          <Badge variant="warning">12 項低庫存</Badge>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardDescription>待處理訂單</CardDescription>
          <CardTitle className="text-3xl">18</CardTitle>
        </CardHeader>
        <CardContent>
          <Badge variant="destructive">3 項逾期</Badge>
        </CardContent>
      </Card>
    </div>
  ),
}
