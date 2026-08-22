/**
 * 捲動接力的事件接線與觸控手勢狀態機（與 React 無關）。
 *
 * 設計背景與各項取捨見 `hooks/useScrollChaining` 的檔頭註解；純 DOM 判定在 `./scrollChaining`。
 * 狀態以 `Session` 顯式傳遞而非包在一層大 closure 裡，讓每支處理器都是獨立的小函數。
 */
import {
  applyScrollWithChaining,
  blocksChaining,
  canScrollFurther,
  findScrollerWithin,
  findScrollerWithRoom,
  isAtAnyEdge,
  wheelDeltaToPixels,
} from './scrollChaining'

/** 觸控放手後的慣性：每幀速度衰減率，與低於此速度即停止的門檻（px/frame） */
const MOMENTUM_DECAY = 0.95
const MOMENTUM_MIN_VELOCITY_PX = 0.5
/** 速度換算基準：一幀約 16ms（60fps） */
const FRAME_MS = 16

/** 進行中的觸控手勢（僅「起手時內層已在邊界」的手勢會建立） */
interface TouchGesture {
  inner: HTMLElement
  lastY: number
  lastMoveAt: number
  velocity: number
  /** 是否已攔下過事件。攔過就不能再交回原生（原生捲動已無法啟動），得全程代管到底 */
  takenOver: boolean
}

/** 單一掛載點的可變狀態 */
interface Session {
  root: HTMLElement
  gesture: TouchGesture | null
  momentumFrame: number | null
  /** touchmove 監聽的 reference：動態掛載 / 卸載都要用同一個 */
  onTouchMove: (event: TouchEvent) => void
}

/** 取出單指觸控點；多指（縮放等）回 null */
function singleTouch(event: TouchEvent): { clientY: number } | null {
  return event.touches.length === 1 ? event.touches[0] : null
}

function stopMomentum(session: Session): void {
  if (session.momentumFrame !== null) cancelAnimationFrame(session.momentumFrame)
  session.momentumFrame = null
}

/** 結束接管：拆掉 non-passive 監聽並清空手勢狀態 */
function endGesture(session: Session): void {
  if (!session.gesture) return
  session.root.removeEventListener('touchmove', session.onTouchMove)
  session.gesture = null
}

function handleWheel(root: HTMLElement, event: WheelEvent): void {
  // ctrl + 滾輪 = 瀏覽器縮放；已被其他處理器接走的事件也不再插手
  if (event.ctrlKey || event.defaultPrevented) return
  // 觸控板左右滑：橫向為主的手勢不接力
  if (Math.abs(event.deltaX) > Math.abs(event.deltaY)) return
  if (event.deltaY === 0) return

  // 邊界判定只看方向（deltaY 正負號與 deltaMode 無關），像素換算留到找出接力對象之後，
  // 才能用該容器自己的 scrollport 高度換算 DOM_DELTA_PAGE。
  const target = event.target instanceof Element ? event.target : null
  const inner = findScrollerWithin(target, root)
  // 游標下沒有內層捲動容器 → 瀏覽器本來就會捲外層，不用插手
  if (!inner || canScrollFurther(inner, event.deltaY)) return
  // 作者以 overscroll-behavior 明示不外傳（下拉選單、對話框內容區等）
  if (blocksChaining(inner)) return

  const outer = findScrollerWithRoom(inner.parentElement, event.deltaY)
  if (!outer) return

  const deltaY = wheelDeltaToPixels(event, outer.clientHeight)
  if (deltaY === 0) return

  // preventDefault 是必要的：未被 latch 的那一發滾輪瀏覽器會自己捲外層，
  // 不擋就會和下面這行疊加成雙倍位移。
  event.preventDefault()
  outer.scrollTop += deltaY
}

function handleTouchStart(session: Session, event: TouchEvent): void {
  stopMomentum(session)
  endGesture(session)
  const touch = singleTouch(event)
  if (!touch) return

  const target = event.target instanceof Element ? event.target : null
  const inner = findScrollerWithin(target, session.root)
  // 只接管「起手時就已在邊界」的手勢。內層還有空間時交給原生：觸控無法中途接管，
  // 硬要接就得全程代管所有捲動，代價是失去原生慣性與 compositor 優化。
  if (!inner || !isAtAnyEdge(inner) || blocksChaining(inner)) return

  // 起手時只知道「在某一端」，不知道使用者要往哪滑——方向要等第一個 move 才明朗，
  // 屆時再複判是否真的需要接管（見 handleTouchMove）。
  session.gesture = {
    inner,
    lastY: touch.clientY,
    lastMoveAt: performance.now(),
    velocity: 0,
    takenOver: false,
  }
  // non-passive 的 touchmove 只在接管期間存在。常駐掛著會讓整個 app 的觸控捲動都必須
  // 等主執行緒（失去 compositor thread 捲動），代價由所有頁面承擔，不划算。
  session.root.addEventListener('touchmove', session.onTouchMove, { passive: false })
}

function handleTouchMove(session: Session, event: TouchEvent): void {
  const gesture = session.gesture
  const touch = singleTouch(event)
  if (!gesture || !touch) return

  const deltaY = gesture.lastY - touch.clientY // 手指上滑 = 內容往下，與 wheel 同號
  const now = performance.now()
  const elapsed = now - gesture.lastMoveAt
  gesture.lastY = touch.clientY
  gesture.lastMoveAt = now
  if (elapsed > 0) gesture.velocity = (deltaY / elapsed) * FRAME_MS
  if (deltaY === 0) return

  // 方向到手才複判：內層在這個方向還捲得動，就沒有接管的理由——趁還沒攔過任何事件，
  // 立刻交回原生（瀏覽器此時仍能啟動捲動，慣性與 compositor 優化都保得住）。
  // 幾乎每個捲動容器初始都停在 scrollTop 0（＝在頂端邊界），少了這道複判會變成
  // 一律代管，反而普遍劣化觸控手感。
  if (!gesture.takenOver && canScrollFurther(gesture.inner, deltaY)) {
    endGesture(session)
    return
  }

  // 攔過就得負責到底：原生捲動已無法中途啟動，方向反轉也繼續由我們分配。
  gesture.takenOver = true
  event.preventDefault()
  applyScrollWithChaining(gesture.inner, deltaY)
}

/** 補上慣性：代管的手勢沒有原生慣性可用，放手後直接停住會明顯比其他頁面遲鈍 */
function startMomentum(session: Session, gesture: TouchGesture): void {
  let velocity = gesture.velocity
  const step = () => {
    session.momentumFrame = null
    velocity *= MOMENTUM_DECAY
    if (Math.abs(velocity) < MOMENTUM_MIN_VELOCITY_PX) return
    if (!applyScrollWithChaining(gesture.inner, velocity)) return
    session.momentumFrame = requestAnimationFrame(step)
  }
  session.momentumFrame = requestAnimationFrame(step)
}

function handleTouchEnd(session: Session): void {
  const gesture = session.gesture
  endGesture(session)
  if (gesture) startMomentum(session, gesture)
}

/**
 * 在指定節點上啟用捲動接力（事件委派，涵蓋其下所有內層捲動容器）。
 *
 * @returns 解除所有監聽與進行中動畫的 cleanup
 */
export function attachScrollChaining(root: HTMLElement): () => void {
  const session: Session = {
    root,
    gesture: null,
    momentumFrame: null,
    onTouchMove: () => {},
  }
  session.onTouchMove = (event) => handleTouchMove(session, event)

  const onWheel = (event: WheelEvent) => handleWheel(root, event)
  const onTouchStart = (event: TouchEvent) => handleTouchStart(session, event)
  const onTouchEnd = () => handleTouchEnd(session)

  // touchstart / touchend 是 passive（零成本）；touchmove 只在接管期間動態掛上，見 handleTouchStart。
  root.addEventListener('wheel', onWheel, { passive: false })
  root.addEventListener('touchstart', onTouchStart, { passive: true })
  root.addEventListener('touchend', onTouchEnd, { passive: true })
  root.addEventListener('touchcancel', onTouchEnd, { passive: true })

  return () => {
    stopMomentum(session)
    endGesture(session)
    root.removeEventListener('wheel', onWheel)
    root.removeEventListener('touchstart', onTouchStart)
    root.removeEventListener('touchend', onTouchEnd)
    root.removeEventListener('touchcancel', onTouchEnd)
  }
}
