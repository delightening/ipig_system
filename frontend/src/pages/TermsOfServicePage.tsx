import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Scale } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export function TermsOfServicePage() {
  const navigate = useNavigate()
  const { t } = useTranslation()

  return (
    <div className="min-h-screen bg-gradient-to-br from-secondary to-background py-12 px-4">
      <div className="mx-auto max-w-3xl">
        <Button
          variant="ghost"
          onClick={() => navigate(-1)}
          className="mb-6 text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          返回
        </Button>

        <Card>
          <CardHeader className="text-center pb-2">
            <Scale className="h-10 w-10 text-muted-foreground/50 mx-auto mb-3" strokeWidth={1.5} />
            <CardTitle className="text-2xl">
              {t('terms.title', '服務條款')}
            </CardTitle>
            <p className="text-sm text-muted-foreground mt-1">
              最後更新日期：2026 年 2 月 28 日
            </p>
          </CardHeader>
          <CardContent className="prose prose-neutral dark:prose-invert max-w-none space-y-6 text-sm leading-relaxed text-foreground/80">
            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">一、服務說明</h3>
              <p>
                豬博士 iPig 實驗動物管理系統（以下簡稱「本系統」）為一套整合式管理平台，提供實驗動物使用計畫書（AUP）管理、動物飼養與觀察紀錄、採購庫存管理、人事出勤管理等功能。本系統僅供經授權之機構人員使用。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">二、使用者責任</h3>
              <p>使用本系統時，您同意：</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li>提供真實、正確之個人資訊與業務資料</li>
                <li>遵守所屬機構之實驗動物使用相關規範與標準作業程序</li>
                <li>不得將帳號提供予他人使用，或使用他人帳號登入</li>
                <li>不得利用本系統從事任何違法或未經授權之行為</li>
                <li>妥善保管登入憑證，如發現帳號遭盜用應立即通報</li>
                <li>不得嘗試規避系統安全措施或存取未經授權之資料</li>
              </ul>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">三、帳號安全</h3>
              <p>
                您的帳號由系統管理員建立與管理。您有責任保護帳號密碼的安全性，並對帳號下之所有活動負責。本系統設有密碼複雜度要求、登入失敗鎖定及閒置逾時等安全機制。首次登入時須變更初始密碼。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">四、智慧財產權</h3>
              <p>
                本系統之軟體、介面設計、原始碼及相關文件之智慧財產權歸系統開發團隊所有。使用者透過本系統建立之業務資料（如計畫書內容、實驗紀錄等）之權利歸屬依所屬機構之相關規定辦理。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">五、服務可用性</h3>
              <p>
                本系統將盡力維持服務之穩定運作，但不保證服務永遠不中斷。因系統維護、更新、不可抗力或其他技術因素導致之服務暫停，將儘可能事先通知使用者。重要資料請自行妥善備份。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">六、責任限制</h3>
              <p>
                在法律允許之最大範圍內，本系統開發團隊不對因使用或無法使用本系統而造成之直接、間接、附帶、特殊或衍生性損害負責。包括但不限於資料遺失、業務中斷或任何其他商業損失。使用者應確保重要資料之備份與驗證。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">七、條款修訂</h3>
              <p>
                本系統保留隨時修訂本服務條款之權利。修訂後之條款將於本頁面公告。繼續使用本系統即視為同意修訂後之條款。重大變更時，將透過系統通知使用者。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-foreground mb-2">八、準據法與管轄</h3>
              <p>
                本服務條款之解釋與適用，以中華民國（台灣）法律為準據法。因本條款所生之爭議，雙方同意以台灣臺北地方法院為第一審管轄法院。
              </p>
            </section>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
