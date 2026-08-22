import type { ImportVetReview } from '@/lib/api/protocol'

type VetItem = ImportVetReview['items'][number]

export const DEFAULT_VET_REVIEW_ITEMS: VetItem[] = [
  '實驗動物之來源(Sources)',
  '實驗方法與程序概況',
  '動物實驗之必要性(含取代、減量、精緻化之3Rs理由)',
  '動物之品種、品系、數量、性別及體重',
  '動物麻醉藥劑、止痛藥、鎮靜藥之名稱、劑量及給藥路徑',
  '動物保定方式、手術程序、術後照顧及對動物預期造成之痛苦或緊迫情形',
  '不預期發病或傷害之處理與安樂死方式及其評估基準(Humane endpoint)',
  '具危險性實驗之防護措施',
  '實驗期限',
  '參與實驗人員',
  '活體採樣或組織採取',
  '其他',
].map((name) => ({ item_name: name, compliance: 'V', comment: '', pi_reply: '' }))
