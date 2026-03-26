import DOMPurify, { type Config } from 'dompurify'

// SEC-H6: 嚴格白名單 — 僅允許手寫簽名所需的 SVG 元素與屬性
// 移除 text/tspan（防止釣魚）、擴充禁止的 event handler
const SVG_CONFIG: Config = {
  USE_PROFILES: { svg: true },
  ADD_TAGS: ['svg', 'path', 'line', 'circle', 'rect', 'polyline', 'polygon', 'ellipse', 'g', 'defs', 'clipPath', 'use'],
  ADD_ATTR: ['d', 'fill', 'stroke', 'stroke-width', 'stroke-linecap', 'stroke-linejoin', 'viewBox', 'xmlns', 'transform', 'opacity', 'cx', 'cy', 'r', 'rx', 'ry', 'x', 'y', 'x1', 'y1', 'x2', 'y2', 'width', 'height', 'points', 'clip-path'],
  FORBID_TAGS: ['script', 'iframe', 'object', 'embed', 'form', 'text', 'tspan', 'foreignObject', 'a', 'image', 'animate', 'set'],
  FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover', 'onmouseout', 'onmousedown', 'onmouseup', 'onfocus', 'onblur', 'onchange', 'onsubmit', 'onkeydown', 'onkeyup', 'onkeypress', 'ontouchstart', 'ontouchend', 'xlink:href', 'href'],
}

export function sanitizeSvg(svg: string): string {
  return DOMPurify.sanitize(svg, SVG_CONFIG) as string
}
