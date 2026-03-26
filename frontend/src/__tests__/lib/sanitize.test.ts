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

  // SEC-H6: 新增嚴格白名單測試
  it('removes text and tspan elements (anti-phishing)', () => {
    const svg = '<svg><text x="10" y="20">Click here</text><tspan>Fake label</tspan></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('<text')
    expect(result).not.toContain('<tspan')
  })

  it('removes foreignObject element', () => {
    const svg = '<svg><foreignObject><body xmlns="http://www.w3.org/1999/xhtml"><div>XSS</div></body></foreignObject></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('foreignObject')
  })

  it('removes anchor and image elements', () => {
    const svg = '<svg><a href="evil.html"><circle r="10"/></a><image href="evil.png"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('<a')
    expect(result).not.toContain('<image')
    expect(result).not.toContain('href')
  })

  it('removes additional event handlers', () => {
    const svg = '<svg><circle r="10" onmouseout="evil()" onkeydown="evil()" ontouchstart="evil()"/></svg>'
    const result = sanitizeSvg(svg)
    expect(result).not.toContain('onmouseout')
    expect(result).not.toContain('onkeydown')
    expect(result).not.toContain('ontouchstart')
  })
})
