import { useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { ArrowLeft, Shield } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export function PrivacyPolicyPage() {
  const navigate = useNavigate()
  const { t } = useTranslation()

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-blue-50 py-12 px-4">
      <div className="mx-auto max-w-3xl">
        <Button
          variant="ghost"
          onClick={() => navigate(-1)}
          className="mb-6 text-slate-600 hover:text-slate-900"
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          返回
        </Button>

        <Card>
          <CardHeader className="text-center pb-2">
            <div className="mx-auto mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-blue-100">
              <Shield className="h-7 w-7 text-blue-600" />
            </div>
            <CardTitle className="text-2xl">
              {t('privacy.title', '隱私權政策')}
            </CardTitle>
            <p className="text-sm text-muted-foreground mt-1">
              最後更新日期：2026 年 2 月 28 日
            </p>
          </CardHeader>
          <CardContent className="prose prose-slate max-w-none space-y-6 text-sm leading-relaxed text-slate-700">
            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">一、前言</h3>
              <p>
                豬博士 iPig 實驗動物管理系統（以下簡稱「本系統」）重視您的隱私權。本政策說明我們如何蒐集、處理、利用及保護您的個人資料，適用於所有使用本系統之使用者。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">二、資料蒐集目的</h3>
              <p>本系統蒐集個人資料之目的包括：</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li>使用者身分驗證與帳號管理</li>
                <li>實驗動物使用計畫書（AUP）之申請、審核與追蹤管理</li>
                <li>實驗動物飼養、觀察、醫療紀錄之建立與維護</li>
                <li>採購、庫存與單據等 ERP 作業流程</li>
                <li>人事出勤、請假等人力資源管理</li>
                <li>系統安全稽核與異常偵測</li>
              </ul>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">三、蒐集之資料類型</h3>
              <p>本系統可能蒐集以下類型之資料：</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li><strong>帳號資料：</strong>姓名、電子郵件、部門、職稱、角色權限</li>
                <li><strong>操作紀錄：</strong>登入時間、操作行為、IP 位址、瀏覽器資訊</li>
                <li><strong>業務資料：</strong>計畫書內容、動物實驗數據、採購單據、出勤紀錄</li>
                <li><strong>系統日誌：</strong>錯誤紀錄、效能監控資料</li>
              </ul>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">四、資料保存期間</h3>
              <p>
                您的個人資料將於蒐集之特定目的存續期間內保存。帳號資料於帳號停用後保留至少一年，以符合稽核需求。實驗動物相關紀錄依法規要求保存至少三年。系統操作日誌保存期間依機構內部資安政策辦理。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">五、資料安全措施</h3>
              <p>本系統採取以下安全措施保護您的資料：</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li>所有傳輸資料均透過 HTTPS/TLS 加密</li>
                <li>密碼以單向雜湊（bcrypt）儲存，不以明文保存</li>
                <li>採用角色存取控制（RBAC）確保最小權限原則</li>
                <li>完整操作稽核紀錄追蹤所有資料變更</li>
              </ul>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">六、使用者權利（資料主體權利）</h3>
              <p>依據個人資料保護法及 GDPR 精神，您享有以下權利：</p>
              <ul className="list-disc pl-5 space-y-1 mt-2">
                <li><strong>存取權與可攜權：</strong>可透過系統「帳號設定」或 API <code className="text-xs bg-slate-100 px-1 rounded">GET /api/v1/me/export</code> 匯出您的個人資料（含帳號、偏好設定、通知設定），取得 JSON 格式複本。</li>
                <li>查詢或請求閱覽您的個人資料</li>
                <li>請求製給個人資料複本</li>
                <li>請求補充或更正個人資料</li>
                <li>請求停止蒐集、處理或利用個人資料</li>
                <li><strong>刪除權：</strong>可透過 API <code className="text-xs bg-slate-100 px-1 rounded">DELETE /api/v1/me/account</code> 申請停用帳號（需重新輸入密碼確認）。帳號將軟刪除並登出所有裝置，如需恢復請聯絡管理員。依法規保存義務之資料仍可能保留。</li>
              </ul>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">七、聯絡方式</h3>
              <p>
                如您對本隱私權政策有任何疑問，或欲行使您的個人資料權利，請聯繫系統管理員或您所屬機構之資料保護聯絡窗口。
              </p>
            </section>

            <section>
              <h3 className="text-base font-semibold text-slate-900 mb-2">八、政策修訂</h3>
              <p>
                本系統保留隨時修訂本隱私權政策之權利。修訂後之政策將於本頁面公告，並以最後更新日期為準。重大變更時，將透過系統通知使用者。
              </p>
            </section>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
