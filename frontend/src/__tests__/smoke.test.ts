import { describe, it, expect } from 'vitest'

describe('Smoke Test', () => {
    it('基本數學運算正確', () => {
        expect(1 + 1).toBe(2)
    })

    it('環境變數可存取', () => {
        // Vite 環境下 import.meta.env 可用
        expect(typeof import.meta.env).toBe('object')
    })
})
