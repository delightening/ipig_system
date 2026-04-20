import React, { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import api, { ProtocolVersion } from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Eye, History } from 'lucide-react'
import { formatDateTime } from '@/lib/utils'
import { useTableSort } from '@/hooks/useTableSort'
import { SortableTableHead } from '@/components/ui/sortable-table-head'
import { ProtocolContentView } from '@/components/protocol/ProtocolContentView'
import { ProtocolComparisonDialog } from '@/components/protocols/ProtocolComparisonDialog'

interface VersionsTabProps {
  protocolId: string
  protocolTitle: string
}

export const VersionsTab = React.memo(function VersionsTab({ protocolId, protocolTitle }: VersionsTabProps) {
  const { t } = useTranslation()
  const { data: versions } = useQuery({
    queryKey: ['protocol-versions', protocolId],
    queryFn: async () => {
      const response = await api.get<ProtocolVersion[]>(`/protocols/${protocolId}/versions`)
      return response.data
    },
    enabled: !!protocolId,
  })

  const { sortedData: sortedVersions, sort, toggleSort } = useTableSort(versions)

  const [compareMode, setCompareMode] = useState(false)
  const [selectedCompareIds, setSelectedCompareIds] = useState<string[]>([])
  const [versionA, setVersionA] = useState<ProtocolVersion | null>(null)
  const [versionB, setVersionB] = useState<ProtocolVersion | null>(null)
  const [comparisonOpen, setComparisonOpen] = useState(false)
  const [showVersionDialog, setShowVersionDialog] = useState(false)
  const [selectedVersion, setSelectedVersion] = useState<ProtocolVersion | null>(null)

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center mb-4">
            <CardDescription>{t('protocols.detail.sections.versionsDesc')}</CardDescription>
            {versions && versions.length >= 2 && (
              <Button
                variant={compareMode ? "destructive" : "outline"}
                size="sm"
                onClick={() => {
                  setCompareMode(!compareMode)
                  setSelectedCompareIds([])
                }}
              >
                {compareMode ? t('common.cancel') : t('protocols.detail.tables.compareVersions')}
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {sortedVersions && sortedVersions.length > 0 ? (
            <div className="space-y-4">
              {compareMode && selectedCompareIds.length === 2 && (
                <div className="bg-status-info-bg border border-blue-100 p-3 rounded-md flex justify-between items-center">
                  <span className="text-sm font-medium text-status-info-text">
                    {t('protocols.detail.tables.twoVersionsSelected')}
                  </span>
                  <Button
                    size="sm"
                    onClick={() => {
                      const vA = sortedVersions.find(v => v.id === selectedCompareIds[0])
                      const vB = sortedVersions.find(v => v.id === selectedCompareIds[1])
                      if (vA && vB) {
                        if (vA.version_no > vB.version_no) {
                          setVersionA(vB)
                          setVersionB(vA)
                        } else {
                          setVersionA(vA)
                          setVersionB(vB)
                        }
                        setComparisonOpen(true)
                      }
                    }}
                  >
                    {t('protocols.detail.tables.startCompare')}
                  </Button>
                </div>
              )}
              <div className="@container">
                <div className="hidden @[600px]:block overflow-x-auto">
                  <Table className="w-full" style={{ minWidth: 360 }}>
                    <TableHeader>
                      <TableRow className="bg-muted/50 hover:bg-muted/50">
                        {compareMode && <TableHead style={{ width: 50 }}></TableHead>}
                        <SortableTableHead style={{ width: 80 }} sortKey="version_no" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.versionNo')}</SortableTableHead>
                        <SortableTableHead style={{ minWidth: 160 }} sortKey="submitted_at" currentSort={sort.column} currentDirection={sort.direction} onSort={toggleSort}>{t('protocols.detail.tables.submitTime')}</SortableTableHead>
                        <TableHead style={{ width: 120 }} className="text-right">{t('protocols.detail.tables.actions')}</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {sortedVersions.map((version) => (
                        <TableRow key={version.id}>
                          {compareMode && (
                            <TableCell style={{ width: 50 }}>
                              <Checkbox
                                checked={selectedCompareIds.includes(version.id)}
                                onCheckedChange={(checked: boolean) => {
                                  if (checked) {
                                    if (selectedCompareIds.length < 2) setSelectedCompareIds([...selectedCompareIds, version.id])
                                  } else {
                                    setSelectedCompareIds(selectedCompareIds.filter(id => id !== version.id))
                                  }
                                }}
                                disabled={!selectedCompareIds.includes(version.id) && selectedCompareIds.length >= 2}
                              />
                            </TableCell>
                          )}
                          <TableCell style={{ width: 80 }} className="font-medium whitespace-nowrap">v{version.version_no}</TableCell>
                          <TableCell style={{ minWidth: 160 }} className="text-xs text-muted-foreground">{formatDateTime(version.submitted_at)}</TableCell>
                          <TableCell style={{ width: 120 }}>
                            <div className="flex items-center justify-end gap-1">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => { setSelectedVersion(version); setShowVersionDialog(true) }}
                              >
                                <Eye className="mr-1 h-4 w-4" />
                                {t('protocols.detail.tables.viewContent')}
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>

                <div className="@[600px]:hidden space-y-3 py-1">
                  {sortedVersions.map((version) => (
                    <div key={version.id} className="rounded-lg border bg-card p-3 flex items-center justify-between gap-2">
                      {compareMode && (
                        <Checkbox
                          checked={selectedCompareIds.includes(version.id)}
                          onCheckedChange={(checked: boolean) => {
                            if (checked) {
                              if (selectedCompareIds.length < 2) setSelectedCompareIds([...selectedCompareIds, version.id])
                            } else {
                              setSelectedCompareIds(selectedCompareIds.filter(id => id !== version.id))
                            }
                          }}
                          disabled={!selectedCompareIds.includes(version.id) && selectedCompareIds.length >= 2}
                        />
                      )}
                      <div className="flex-1">
                        <div className="text-sm font-semibold text-foreground">v{version.version_no}</div>
                        <div className="text-xs text-muted-foreground">{formatDateTime(version.submitted_at)}</div>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => { setSelectedVersion(version); setShowVersionDialog(true) }}
                      >
                        <Eye className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>

              </div>
            </div>
          ) : (
            <div className="flex flex-col items-center gap-2 py-10 text-muted-foreground">
              <History className="h-8 w-8" />
              <p className="text-sm">{t('protocols.detail.tables.noVersions')}</p>
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={showVersionDialog} onOpenChange={setShowVersionDialog}>
        <DialogContent className="max-w-3xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>{t('protocols.detail.dialogs.version.title', { version: selectedVersion?.version_no })}</DialogTitle>
            <DialogDescription>
              {selectedVersion && t('protocols.detail.dialogs.version.submitted', { time: formatDateTime(selectedVersion.submitted_at) })}
            </DialogDescription>
          </DialogHeader>
          <div className="overflow-auto max-h-[60vh] py-4">
            {selectedVersion?.content_snapshot ? (
              <ProtocolContentView
                workingContent={selectedVersion.content_snapshot}
                protocolTitle={protocolTitle}
              />
            ) : (
              <p className="text-center text-muted-foreground py-8">{t('protocols.detail.dialogs.version.noContent')}</p>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowVersionDialog(false)}>
              {t('common.close')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {versionA && versionB && (
        <ProtocolComparisonDialog
          open={comparisonOpen}
          onOpenChange={setComparisonOpen}
          versionA={{
            version_no: versionA.version_no,
            content: versionA.content_snapshot as unknown as import('@/types/protocol').ProtocolWorkingContent
          }}
          versionB={{
            version_no: versionB.version_no,
            content: versionB.content_snapshot as unknown as import('@/types/protocol').ProtocolWorkingContent
          }}
          protocolTitle={protocolTitle}
        />
      )}
    </>
  )
})
