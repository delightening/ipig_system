import DOMPurify, { type Config } from 'dompurify'

const SVG_CONFIG: Config = {
  USE_PROFILES: { svg: true, svgFilters: true },
  ADD_TAGS: ['svg', 'path', 'line', 'circle', 'rect', 'polyline', 'polygon', 'ellipse', 'g', 'defs', 'clipPath', 'use', 'text', 'tspan'],
  ADD_ATTR: ['d', 'fill', 'stroke', 'stroke-width', 'stroke-linecap', 'stroke-linejoin', 'viewBox', 'xmlns', 'transform', 'opacity', 'cx', 'cy', 'r', 'rx', 'ry', 'x', 'y', 'x1', 'y1', 'x2', 'y2', 'width', 'height', 'points', 'clip-path', 'font-size', 'text-anchor'],
  FORBID_TAGS: ['script', 'iframe', 'object', 'embed', 'form'],
  FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover', 'onfocus', 'onblur'],
}

export function sanitizeSvg(svg: string): string {
  return DOMPurify.sanitize(svg, SVG_CONFIG) as string
}
