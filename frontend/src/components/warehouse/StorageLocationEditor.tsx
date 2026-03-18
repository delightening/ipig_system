import { useMemo, useRef, useState, useEffect } from 'react'
import {
    StorageLocationWithWarehouse,
    StorageLocationType,
    StorageLayoutItem,
    storageLocationTypeNames,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
    Loader2,
    Lock,
    Unlock,
    Save,
    Plus,
    Package,
    DoorOpen,
    Square,
} from 'lucide-react'
import ReactGridLayout, { WidthProvider } from 'react-grid-layout/legacy'
import 'react-grid-layout/css/styles.css'
import 'react-resizable/css/styles.css'

const GridLayout = WidthProvider(ReactGridLayout)

const COLS = 12
const MARGIN = 12
const DEFAULT_ROW_HEIGHT = 60
const MIN_ROW_HEIGHT = 30

// 預設顏色
const DEFAULT_COLORS: Record<StorageLocationType, string> = {
    shelf: '#3b82f6',  // blue
    rack: '#10b981',   // green
    zone: '#f59e0b',   // amber
    bin: '#6366f1',    // indigo
    wall: '#475569',   // slate-600
    door: '#94a3b8',   // slate-400
    window: '#bae6fd', // sky-200
}

interface GridLayoutItem {
    i: string
    x: number
    y: number
    w: number
    h: number
    minW?: number
    minH?: number
}

interface StorageLocationEditorProps {
    locations: StorageLocationWithWarehouse[]
    isLoading: boolean
    isEditMode: boolean
    setIsEditMode: (v: boolean | ((v: boolean) => boolean)) => void
    onLayoutChange: (layout: StorageLayoutItem[]) => void
    onSaveLayout: () => void
    isSavingLayout: boolean
    hasUnsavedChanges: boolean
    onAddLocationClick: () => void
    selectedLocationId: string | null
    onLocationClick: (loc: StorageLocationWithWarehouse) => void
}

export function StorageLocationEditor({
    locations,
    isLoading,
    isEditMode,
    setIsEditMode,
    onLayoutChange,
    onSaveLayout,
    isSavingLayout,
    hasUnsavedChanges,
    onAddLocationClick,
    selectedLocationId,
    onLocationClick,
}: StorageLocationEditorProps) {
    const containerRef = useRef<HTMLDivElement>(null)
    const [rowHeight, setRowHeight] = useState(DEFAULT_ROW_HEIGHT)

    useEffect(() => {
        const el = containerRef.current
        if (!el) return
        const observer = new ResizeObserver((entries) => {
            const width = entries[0].contentRect.width
            const colWidth = (width - MARGIN * (COLS - 1)) / COLS
            setRowHeight(Math.max(Math.round(colWidth), MIN_ROW_HEIGHT))
        })
        observer.observe(el)
        return () => observer.disconnect()
    }, [])

    // 轉換為 react-grid-layout 格式
    const gridLayout: GridLayoutItem[] = useMemo(() => {
        return locations.map((loc) => ({
            i: loc.id,
            x: loc.col_index,
            y: loc.row_index,
            w: loc.width || 2,
            h: loc.height || 2,
            minW: 1,
            minH: 1,
        }))
    }, [locations])

    const handleInternalLayoutChange = (newLayout: Array<{ i: string; x: number; y: number; w: number; h: number }>) => {
        if (!isEditMode) return
        const items: StorageLayoutItem[] = newLayout.map((item) => ({
            id: item.i,
            row_index: item.y,
            col_index: item.x,
            width: item.w,
            height: item.h,
        }))
        onLayoutChange(items)
    }

    if (isLoading) {
        return (
            <div className="flex items-center justify-center py-16 border rounded-lg bg-slate-50 min-h-[480px]">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
            </div>
        )
    }

    return (
        <div className="space-y-4">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <h3 className="text-lg font-semibold">儲位佈局圖</h3>
                    {isEditMode && (
                        <Badge variant="outline" className="text-blue-600 border-blue-200 bg-blue-50">
                            編輯模式
                        </Badge>
                    )}
                </div>
                <div className="flex items-center gap-2">
                    <Button
                        size="sm"
                        onClick={onAddLocationClick}
                        className="bg-blue-600 hover:bg-blue-700 text-white"
                    >
                        <Plus className="h-4 w-4 mr-1" />
                        新增儲位
                    </Button>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setIsEditMode((v) => !v)}
                    >
                        {isEditMode ? (
                            <>
                                <Lock className="h-4 w-4 mr-1" />
                                鎖定佈局
                            </>
                        ) : (
                            <>
                                <Unlock className="h-4 w-4 mr-1" />
                                解鎖佈局
                            </>
                        )}
                    </Button>
                    {isEditMode && hasUnsavedChanges && (
                        <Button
                            size="sm"
                            onClick={onSaveLayout}
                            disabled={isSavingLayout}
                        >
                            {isSavingLayout && (
                                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                            )}
                            <Save className="h-4 w-4 mr-1" />
                            儲存佈局
                        </Button>
                    )}
                </div>
            </div>

            {isEditMode && (
                <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg text-xs text-blue-700">
                    📐 編輯模式啟用中：拖拽方塊調整位置，拖拽右下角調整大小。建築結構（牆、門、窗）也在此調整。
                </div>
            )}

            <div ref={containerRef} className="border rounded-lg bg-slate-50 p-4 min-h-[480px] relative">
                {locations.length > 0 ? (
                    <GridLayout
                        className="layout"
                        layout={gridLayout}
                        cols={COLS}
                        rowHeight={rowHeight}
                        onLayoutChange={(layout) => handleInternalLayoutChange([...layout])}
                        isDraggable={isEditMode}
                        isResizable={isEditMode}
                        margin={[MARGIN, MARGIN]}
                        containerPadding={[0, 0]}
                        useCSSTransforms={true}
                        autoSize={true}
                        compactType={null}
                    >
                        {locations.map((loc) => {
                            const isSelected = selectedLocationId === loc.id
                            const isStructural = ['wall', 'door', 'window'].includes(loc.location_type)
                            
                            // 根據類型決定圖示
                            const getIcon = () => {
                                if (loc.location_type === 'door') return <DoorOpen className="h-5 w-5 opacity-40" />
                                if (loc.location_type === 'wall') return <Square className="h-5 w-5 opacity-20" />
                                if (loc.location_type === 'window') return <Square className="h-5 w-5 opacity-40 text-blue-400" />
                                return null
                            }

                            // 建築結構特殊樣式
                            const structuralStyles: React.CSSProperties = {}
                            if (loc.location_type === 'window') {
                                structuralStyles.backgroundImage = 'repeating-linear-gradient(45deg, transparent, transparent 5px, rgba(255,255,255,0.2) 5px, rgba(255,255,255,0.2) 10px)'
                            } else if (loc.location_type === 'door') {
                                structuralStyles.borderRadius = '4px 4px 40px 4px'
                            }

                            return (
                                <div
                                    key={loc.id}
                                    className={`rounded shadow-sm overflow-hidden transition-all ${
                                        isEditMode
                                            ? 'cursor-move ring-1 ring-blue-200'
                                            : isStructural
                                              ? 'pointer-events-none opacity-80' // 結構在鎖定模式下不可點擊
                                              : 'cursor-pointer hover:shadow-md'
                                    } ${
                                        isSelected && !isEditMode
                                            ? 'ring-2 ring-offset-2 ring-blue-500 z-10 scale-[1.02]'
                                            : ''
                                    }`}
                                    style={{
                                        backgroundColor: loc.color || DEFAULT_COLORS[loc.location_type],
                                        border: isSelected && !isEditMode ? '2px solid white' : 'none',
                                        ...structuralStyles,
                                    }}
                                    onClick={() => {
                                        if (isEditMode || isStructural) return
                                        onLocationClick(loc)
                                    }}
                                >
                                    <div className="h-full p-2 flex flex-col justify-between text-white overflow-hidden relative">
                                        {/* 背景裝飾圖示 */}
                                        <div className="absolute right-1 bottom-1">
                                            {getIcon()}
                                        </div>

                                        <div className="relative z-10">
                                            {!isStructural && (
                                                <>
                                                    <div className="font-bold text-sm truncate leading-tight">
                                                        {loc.name || loc.code}
                                                    </div>
                                                    <div className="text-[10px] opacity-80 truncate uppercase tracking-tighter">
                                                        {loc.code}
                                                    </div>
                                                </>
                                            )}
                                            {isStructural && (
                                                <>
                                                    {loc.name && (
                                                        <div className="font-bold text-sm truncate leading-tight">
                                                            {loc.name}
                                                        </div>
                                                    )}
                                                    <div className="text-[10px] font-medium opacity-60">
                                                        {storageLocationTypeNames[loc.location_type]}
                                                    </div>
                                                </>
                                            )}
                                        </div>

                                        {!isStructural && (
                                            <div className="flex items-center justify-between mt-auto relative z-10">
                                                <div className="flex items-center gap-0.5 text-[10px] bg-black/10 px-1 rounded">
                                                    <Package className="h-2.5 w-2.5" />
                                                    <span>
                                                        {loc.current_count}
                                                        {loc.capacity && `/${loc.capacity}`}
                                                    </span>
                                                </div>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            )
                        })}
                    </GridLayout>
                ) : (
                    <div className="flex flex-col items-center justify-center h-full text-muted-foreground bg-white/50 rounded-lg">
                        <Package className="h-12 w-12 mb-2 opacity-20" />
                        <p className="text-sm">尚未建立任何儲位或結構</p>
                    </div>
                )}
            </div>
        </div>
    )
}
