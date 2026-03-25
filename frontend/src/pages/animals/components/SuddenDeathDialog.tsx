import React from 'react'

import { Button } from '@/components/ui/button'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Loader2, Zap } from 'lucide-react'

import type { SuddenDeathFormData } from '../hooks/useAnimalDetailMutations'

interface SuddenDeathDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  earTag: string
  form: SuddenDeathFormData
  onFormChange: React.Dispatch<React.SetStateAction<SuddenDeathFormData>>
  isPending: boolean
  onConfirm: () => void
}

export function SuddenDeathDialog({
  open,
  onOpenChange,
  earTag,
  form,
  onFormChange,
  isPending,
  onConfirm,
}: SuddenDeathDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-destructive">
            <Zap className="h-5 w-5" />
            {'\u767B\u8A18\u731D\u6B7B \u2014 \u8033\u865F'} {earTag}
          </DialogTitle>
          <DialogDescription>
            {'\u767B\u8A18\u5F8C\u52D5\u7269\u72C0\u614B\u5C07\u81EA\u52D5\u66F4\u65B0\u70BA\u300C\u731D\u6B7B\u300D\uFF0C\u6B64\u64CD\u4F5C\u4E0D\u53EF\u5FA9\u539F\u3002'}
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="space-y-2">
            <Label htmlFor="sd-discovered-at">{'\u767C\u73FE\u6642\u9593'} *</Label>
            <Input
              id="sd-discovered-at"
              type="datetime-local"
              value={form.discovered_at}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, discovered_at: e.target.value }))
              }
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="sd-location">{'\u767C\u73FE\u5730\u9EDE'}</Label>
            <Input
              id="sd-location"
              placeholder={'\u5982\uFF1AA01 \u6B04\u4F4D'}
              value={form.location}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, location: e.target.value }))
              }
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="sd-probable-cause">{'\u53EF\u80FD\u539F\u56E0'}</Label>
            <Textarea
              id="sd-probable-cause"
              placeholder={'\u63CF\u8FF0\u53EF\u80FD\u7684\u6B7B\u56E0...'}
              value={form.probable_cause}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, probable_cause: e.target.value }))
              }
              className="min-h-[80px]"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="sd-remark">{'\u5099\u8A3B'}</Label>
            <Textarea
              id="sd-remark"
              placeholder={'\u5176\u4ED6\u5099\u8A3B...'}
              value={form.remark}
              onChange={(e) =>
                onFormChange((prev) => ({ ...prev, remark: e.target.value }))
              }
              className="min-h-[60px]"
            />
          </div>
          <div className="flex items-center space-x-2">
            <input
              id="sd-requires-pathology"
              type="checkbox"
              aria-label={'\u9700\u8981\u75C5\u7406\u6AA2\u67E5'}
              checked={form.requires_pathology}
              onChange={(e) =>
                onFormChange((prev) => ({
                  ...prev,
                  requires_pathology: e.target.checked,
                }))
              }
              className="h-4 w-4 rounded border-border text-destructive focus:ring-destructive"
            />
            <Label
              htmlFor="sd-requires-pathology"
              className="text-sm font-normal cursor-pointer"
            >
              {'\u9700\u8981\u75C5\u7406\u6AA2\u67E5'}
            </Label>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {'\u53D6\u6D88'}
          </Button>
          <Button
            className="bg-destructive hover:bg-destructive/90 text-white"
            disabled={!form.discovered_at || isPending}
            onClick={onConfirm}
          >
            {isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {'\u78BA\u8A8D\u767B\u8A18\u731D\u6B7B'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
