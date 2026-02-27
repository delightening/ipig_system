import { useState } from 'react'
import { AnimalSacrifice } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Plus, Edit2, Heart } from 'lucide-react'
import { SacrificeFormDialog } from './SacrificeFormDialog'

interface SacrificeTabProps {
  animalId: string
  earTag: string
  sacrifice: AnimalSacrifice | undefined
}

export function SacrificeTab({ animalId, earTag, sacrifice }: SacrificeTabProps) {
  const [showDialog, setShowDialog] = useState(false)

  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>犧牲/採樣紀錄</CardTitle>
          <CardDescription>記錄實驗結束後的犧牲與採樣資訊</CardDescription>
        </CardHeader>
        <CardContent>
          {!sacrifice ? (
            <div className="text-center py-12 text-slate-500">
              <Heart className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無犧牲/採樣紀錄</p>
              <Button
                className="mt-4 bg-purple-600 hover:bg-purple-700 text-white"
                onClick={() => setShowDialog(true)}
              >
                <Plus className="h-4 w-4 mr-2" />
                建立紀錄
              </Button>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-slate-500">犧牲日期</Label>
                  <p className="font-medium">
                    {sacrifice.sacrifice_date
                      ? new Date(sacrifice.sacrifice_date).toLocaleDateString('zh-TW')
                      : '-'
                    }
                  </p>
                </div>
                <div>
                  <Label className="text-slate-500">確定犧牲</Label>
                  <p className="font-medium">
                    {sacrifice.confirmed_sacrifice ? (
                      <Badge className="bg-red-100 text-red-800">已確認</Badge>
                    ) : '否'}
                  </p>
                </div>
                <div>
                  <Label className="text-slate-500">Zoletil-50 (ml)</Label>
                  <p className="font-medium">{sacrifice.zoletil_dose || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">200V電擊</Label>
                  <p className="font-medium">{sacrifice.method_electrocution ? '是' : '否'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">放血</Label>
                  <p className="font-medium">{sacrifice.method_bloodletting ? '是' : '否'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">其他方式</Label>
                  <p className="font-medium">{sacrifice.method_other || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">採樣</Label>
                  <p className="font-medium">{sacrifice.sampling || '-'}</p>
                </div>
                <div>
                  <Label className="text-slate-500">血液採樣 (ml)</Label>
                  <p className="font-medium">{sacrifice.blood_volume_ml || '-'}</p>
                </div>
              </div>
              <div className="flex justify-end">
                <Button
                  className="bg-purple-600 hover:bg-purple-700 text-white"
                  onClick={() => setShowDialog(true)}
                >
                  <Edit2 className="h-4 w-4 mr-2" />
                  編輯
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <SacrificeFormDialog
        open={showDialog}
        onOpenChange={setShowDialog}
        animalId={animalId}
        earTag={earTag}
        sacrifice={sacrifice || undefined}
      />
    </>
  )
}
