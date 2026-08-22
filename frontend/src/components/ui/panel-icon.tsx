/**
 * PanelIcon 元件
 * 根據 icon 值判斷是 emoji 或 SVG 路徑，自動選擇渲染方式
 */

interface PanelIconProps {
    icon: string | null | undefined
    fallback?: string
    className?: string
    size?: number
}

export function PanelIcon({ icon, fallback = '📋', className = '', size = 18 }: PanelIconProps) {
    const value = icon || fallback

    // 判斷是否為路徑（以 / 開頭）
    if (value.startsWith('/')) {
        return (
            <img
                src={value}
                alt=""
                width={size}
                height={size}
                className={`inline-block ${className}`}
                style={{ verticalAlign: 'middle' }}
            />
        )
    }

    // 否則當作 emoji 渲染
    return <span className={className}>{value}</span>
}
