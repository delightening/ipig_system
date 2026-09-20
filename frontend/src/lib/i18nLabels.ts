import i18n from '@/lib/i18n'

/**
 * 「代碼 → 顯示文字」對照表的 i18n 版本（getter-based）。
 *
 * 為什麼不是頂層 `t()`：模組級常數在 import 當下就求值，之後切語言不會更新。
 * 這裡對每個代碼定義 **getter**，`map[code]` 每次讀取才呼叫 `i18n.t()`，
 * 所以語言切換後的下一次 render（元件本身有 `useTranslation` 訂閱）讀到的就是新語言，
 * 不必重建對照表，消費端也不必改寫。
 *
 * 行為與一般物件相同的部分：`Object.keys / Object.entries / Object.values / in / for...in`
 * 都可用（屬性是 enumerable），未知代碼讀到 `undefined`。
 *
 * ⚠️ 會**凍結**值的用法：`{ ...map }`、`Object.assign({}, map)`、`structuredClone(map)`、
 * 在模組頂層把 `Object.values(map)` 存進常數——這些會在當下把 getter 求值成字串快照，
 * 之後切語言不會更新。要快照請在 render 期間、每次重新取。
 *
 * @param keyPrefix 語言包鍵前綴，實際鍵為 `${keyPrefix}.${code}`
 * @param codes 要建立的代碼集合（順序即 `Object.keys` 順序）
 * @param fallbacks 個別代碼在**兩個語言包都缺鍵**時的後備字串；預設回傳代碼本身，
 *   不放中文預設值（en 缺鍵會先落到 fallbackLng 的 zh-TW，不會走到這裡）
 */
export function createLabelMap<K extends string>(
    keyPrefix: string,
    codes: readonly K[],
    fallbacks?: Partial<Record<K, string>>,
): Record<K, string> {
    const map = {} as Record<K, string>
    for (const code of codes) {
        Object.defineProperty(map, code, {
            enumerable: true,
            configurable: true,
            get: () => translateLabel(keyPrefix, code, fallbacks?.[code]),
        })
    }
    return map
}

/** `{ value, label }` 選項陣列的 getter 版本：`value` 是固定代碼，`label` 每次讀取才翻譯。 */
export interface LabelOption<V extends string> {
    readonly value: V
    readonly label: string
}

/**
 * 建立 `{ value, label }[]` 選項陣列（給 `.map((o) => ...)` 型的消費端），`label` 為 getter。
 * 鍵規則與 {@link createLabelMap} 相同：`${keyPrefix}.${value}`。
 * `value` 是送後端／存 DB 的代碼，**不翻**。
 */
export function createLabelOptions<V extends string>(
    keyPrefix: string,
    values: readonly V[],
    fallbacks?: Partial<Record<V, string>>,
): readonly LabelOption<V>[] {
    return values.map((value) => {
        const option = { value } as { value: V; label: string }
        Object.defineProperty(option, 'label', {
            enumerable: true,
            configurable: true,
            get: () => translateLabel(keyPrefix, value, fallbacks?.[value]),
        })
        return option
    })
}

function translateLabel(keyPrefix: string, code: string, fallback: string | undefined): string {
    return i18n.t(`${keyPrefix}.${code}`, { defaultValue: fallback ?? code })
}
