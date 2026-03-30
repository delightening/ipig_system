import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useQueryClient } from '@tanstack/react-query'
import { Animal, allAnimalStatusNames, animalBreedNames, animalGenderNames, AnimalStatus, animalFieldCorrectionApi } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { Edit2, FileEdit } from 'lucide-react'
import { RequestCorrectionDialog } from './RequestCorrectionDialog'

const getPenLocationDisplay = (animal: { status: AnimalStatus; pen_location?: string | null }) => {
  if (animal.status === 'completed' && !animal.pen_location) {
    return '犧牲'
  }
  return animal.pen_location || '-'
}

interface AnimalInfoTabProps {
  animal: Animal
}

export function AnimalInfoTab({ animal }: AnimalInfoTabProps) {
  const [correctionDialogOpen, setCorrectionDialogOpen] = useState(false)
  const queryClient = useQueryClient()

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>動物資料</CardTitle>
          <CardDescription>動物基本資料</CardDescription>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => setCorrectionDialogOpen(true)}
            className="text-status-warning-text border-status-warning-border hover:bg-status-warning-bg"
          >
            <FileEdit className="h-4 w-4 mr-2" />
            申請修正
          </Button>
          <Button className="bg-purple-600 hover:bg-purple-700 text-white" asChild>
            <Link to={`/animals/${animal.id}/edit`}>
              <Edit2 className="h-4 w-4 mr-2" />
              編輯
            </Link>
          </Button>
        </div>
      </CardHeader>

      <RequestCorrectionDialog
        open={correctionDialogOpen}
        onOpenChange={setCorrectionDialogOpen}
        animal={animal}
        onSubmit={async (data) => {
          await animalFieldCorrectionApi.create(animal.id, data)
          queryClient.invalidateQueries({ queryKey: ['animal', animal.id] })
          toast({ title: '成功', description: '修正申請已提交，待管理員審核' })
        }}
      />
      <CardContent>
        <div className="grid grid-cols-2 gap-x-8 gap-y-4">
          <div>
            <Label className="text-muted-foreground">耳號</Label>
            <p className="font-medium">{animal.ear_tag}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">動物狀態</Label>
            <p className="font-medium">{allAnimalStatusNames[animal.status]}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">進場日期</Label>
            <p className="font-medium">{new Date(animal.entry_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' })}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">品種</Label>
            <p className="font-medium">{animalBreedNames[animal.breed]}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">來源</Label>
            <p className="font-medium">{animal.source_name || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">進場體重 (kg)</Label>
            <p className="font-medium">{animal.entry_weight || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">性別</Label>
            <p className="font-medium">{animalGenderNames[animal.gender]}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">出生日期</Label>
            <p className="font-medium">
              {animal.birth_date ? new Date(animal.birth_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}
            </p>
          </div>
          <div>
            <Label className="text-muted-foreground">實驗前代號</Label>
            <p className="font-medium">{animal.pre_experiment_code || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">IACUC No.</Label>
            <p className="font-medium">{animal.iacuc_no || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">實驗日期</Label>
            <p className="font-medium">
              {animal.experiment_date ? new Date(animal.experiment_date).toLocaleDateString('zh-TW', { timeZone: 'Asia/Taipei' }) : '-'}
            </p>
          </div>
          <div>
            <Label className="text-muted-foreground">欄位</Label>
            <p className="font-medium">{getPenLocationDisplay(animal)}</p>
          </div>
          <div className="col-span-2">
            <Label className="text-muted-foreground">備註</Label>
            <p className="font-medium">{animal.remark || '-'}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">系統號</Label>
            <p className="font-medium" title={animal.id}>{animal.id.slice(0, 8)}</p>
          </div>
          <div>
            <Label className="text-muted-foreground">建立時間</Label>
            <p className="font-medium">{new Date(animal.created_at).toLocaleString('zh-TW', { timeZone: 'Asia/Taipei' })}</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
