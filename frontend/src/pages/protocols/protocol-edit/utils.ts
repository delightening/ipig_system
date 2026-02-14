// ProtocolEditPage 工具函數
// 包含 deepMerge 和 updateWorkingContent 邏輯

import type { FormData } from './constants'

/**
 * 深層合併物件（陣列取 source 值）
 */
export function deepMerge(target: any, source: any): any {
    if (typeof target !== 'object' || target === null ||
        typeof source !== 'object' || source === null) {
        return source;
    }

    if (Array.isArray(target) && Array.isArray(source)) {
        return source;
    }

    if (Array.isArray(target) || Array.isArray(source)) {
        return source;
    }

    const output = { ...target };
    Object.keys(source).forEach(key => {
        if (Object.prototype.hasOwnProperty.call(source, key)) {
            if (key in target) {
                output[key] = deepMerge(target[key], source[key]);
            } else {
                output[key] = source[key];
            }
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
