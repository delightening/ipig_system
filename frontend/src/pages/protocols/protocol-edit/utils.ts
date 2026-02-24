// ProtocolEditPage 工具函數
// 包含 deepMerge 和 updateWorkingContent 邏輯

import type { FormData } from './constants'

/**
 * 深層合併物件（陣列取 source 值）
 */
export function deepMerge(target: any, source: any): any {
    // 如果 source 不是物件或是 null，直接回傳 source
    if (typeof source !== 'object' || source === null) {
        return source;
    }

    // 如果 target 不是物件或是 null，為了保持結構，回傳 source 的副本
    if (typeof target !== 'object' || target === null) {
        return Array.isArray(source) ? [...source] : { ...source };
    }

    // 處理陣列：直接使用 source 的內容（通常 AUP 內容中的陣列是整組替換）
    if (Array.isArray(source)) {
        return [...source];
    }

    // 處理物件
    const output = { ...target };
    Object.keys(source).forEach(key => {
        const sourceValue = source[key];
        const targetValue = target[key];

        if (typeof sourceValue === 'object' && sourceValue !== null) {
            // 如果 target 中沒有該 key，或者 target[key] 不是物件，則直接複製 sourceValue
            if (!(key in target) || typeof targetValue !== 'object' || targetValue === null) {
                output[key] = Array.isArray(sourceValue) ? [...sourceValue] : deepMerge({}, sourceValue);
            } else {
                // 遞迴合併
                output[key] = deepMerge(targetValue, sourceValue);
            }
        } else {
            // 基本型別直接覆蓋
            output[key] = sourceValue;
        }
    });

    return output;
}

/**
 * 更新 working_content 中指定 section 的某個 path
 */
export function updateWorkingContent(
    setFormData: React.Dispatch<React.SetStateAction<FormData>>,
    section: keyof FormData['working_content'],
    path: string,
    value: any
) {
    setFormData((prev) => {
        const newContent = { ...prev.working_content }
        const sectionData = { ...(newContent[section] as any) }
        if (path.includes('.')) {
            const parts = path.split('.')
            let current = sectionData
            for (let i = 0; i < parts.length - 1; i++) {
                current[parts[i]] = { ...current[parts[i]] }
                current = current[parts[i]]
            }
            current[parts[parts.length - 1]] = value
        } else {
            sectionData[path] = value
        }

        newContent[section] = sectionData
        return { ...prev, working_content: newContent }
    })
}
