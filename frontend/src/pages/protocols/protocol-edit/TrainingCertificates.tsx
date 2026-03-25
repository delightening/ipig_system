import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface TrainingCertificatesProps {
  trainingCode: string
  certificates: Array<{ training_code: string; certificate_no: string }>
  allCertificates: Array<{ training_code: string; certificate_no: string }>
  onCertificatesChange: (certs: Array<{ training_code: string; certificate_no: string }>) => void
  t: (key: string) => string
}

export function TrainingCertificates({
  trainingCode,
  certificates,
  allCertificates,
  onCertificatesChange,
  t,
}: TrainingCertificatesProps) {
  const findGlobalIndex = (certIndex: number) => {
    let count = 0
    for (let i = 0; i < allCertificates.length; i++) {
      if (allCertificates[i].training_code === trainingCode) {
        if (count === certIndex) return i
        count++
      }
    }
    return -1
  }

  return (
    <div className="space-y-1 pl-4 border-l-2 border-border">
      <Label className="text-xs font-semibold">{trainingCode}:</Label>
      {certificates.map((cert, certIndex) => {
        const globalCertIndex = findGlobalIndex(certIndex)
        return (
          <div key={certIndex} className="flex items-center gap-2">
            <Input
              value={cert.certificate_no}
              onChange={(e) => {
                const newCerts = [...allCertificates]
                if (globalCertIndex >= 0 && globalCertIndex < newCerts.length) {
                  newCerts[globalCertIndex] = { ...newCerts[globalCertIndex], certificate_no: e.target.value }
                  onCertificatesChange(newCerts)
                }
              }}
              placeholder={t('aup.personnel.addDialog.placeholders.certNo')}
              className="text-xs h-7"
            />
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-6 w-6 text-destructive"
              aria-label="刪除"
              onClick={() => {
                const newCerts = [...allCertificates]
                if (globalCertIndex >= 0 && globalCertIndex < newCerts.length) {
                  newCerts.splice(globalCertIndex, 1)
                  onCertificatesChange(newCerts)
                }
              }}
            >
              X
            </Button>
          </div>
        )
      })}
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="h-7 text-xs"
        onClick={() => {
          onCertificatesChange([
            ...allCertificates,
            { training_code: trainingCode, certificate_no: '' },
          ])
        }}
      >
        + {t('aup.personnel.addDialog.buttons.addCert')}
      </Button>
    </div>
  )
}
