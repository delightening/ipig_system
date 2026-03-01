import { describe, it, expect } from 'vitest'
import { sanitizeSvg } from '@/lib/sanitize'

describe('sanitizeSvg', () => {
  it('allows valid SVG elements', () => {
    const svg = '<svg viewBox="0 0 100 100"><path d="M10 10 L90 90" fill="none" stroke="black"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).toContain('<svg')
    expect(result).toContain('<path')
  })

  it('allows common SVG attributes', () => {
    const svg = '<svg xmlns="http://www.w3.org/2000/svg"><circle cx="50" cy="50" r="40" fill="red"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).toContain('cx="50"')
    expect(result).toContain('cy="50"')
    expect(result).toContain('r="40"')
  })

  it('removes script tags', () => {
    const svg = '<svg><script>alert("xss")</script><circle cx="50" cy="50" r="40"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('<script')
    expect(result).not.toContain('alert')
  })

  it('removes iframe tags', () => {
    const svg = '<svg><iframe src="evil.html"></iframe></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('<iframe')
  })

  it('removes event handler attributes', () => {
    const svg = '<svg><circle cx="50" cy="50" r="40" onclick="alert(1)" onerror="evil()"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('onclick')
    expect(result).not.toContain('onerror')
  })

  it('removes onload attribute', () => {
    const svg = '<svg onload="alert(1)"><rect width="100" height="100"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('onload')
  })

  it('preserves complex SVG structures', () => {
    const svg = '<svg><g transform="translate(10,10)"><rect x="0" y="0" width="50" height="50"/></g></svg>'
    const result = sanitizeSvg(svg)
    expect(result).toContain('<g')
    expect(result).toContain('transform')
    expect(result).toContain('<rect')
  })

  it('handles empty string', () => {
    expect(sanitizeSvg('')).toBe('')
  })
})
