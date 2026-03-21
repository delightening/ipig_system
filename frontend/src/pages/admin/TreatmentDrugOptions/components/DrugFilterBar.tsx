import { Input } from '@/components/ui/input'
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select'
import { DRUG_CATEGORIES } from '@/types/treatment-drug'
import { Search } from 'lucide-react'

interface DrugFilterBarProps {
    keyword: string
    onKeywordChange: (value: string) => void
    filterCategory: string
    onCategoryChange: (value: string) => void
    filterActive: string
    onActiveChange: (value: string) => void
}

export function DrugFilterBar({
    keyword,
    onKeywordChange,
    filterCategory,
    onCategoryChange,
    filterActive,
    onActiveChange,
}: DrugFilterBarProps) {
    return (
        <div className="flex flex-wrap gap-3 items-center bg-white p-4 rounded-lg border">
            <div className="relative flex-1 min-w-[200px]">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-slate-400" />
                <Input
                    placeholder="搜尋藥物名稱..."
                    value={keyword}
                    onChange={(e) => onKeywordChange(e.target.value)}
                    className="pl-9"
                />
            </div>
            <Select value={filterCategory} onValueChange={onCategoryChange}>
                <SelectTrigger className="w-[140px]">
                    <SelectValue placeholder="分類" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="all">全部分類</SelectItem>
                    {DRUG_CATEGORIES.map((cat) => (
                        <SelectItem key={cat} value={cat}>{cat}</SelectItem>
                    ))}
                </SelectContent>
            </Select>
            <Select value={filterActive} onValueChange={onActiveChange}>
                <SelectTrigger className="w-[120px]">
                    <SelectValue placeholder="狀態" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="all">全部</SelectItem>
                    <SelectItem value="active">啟用中</SelectItem>
                    <SelectItem value="inactive">已停用</SelectItem>
                </SelectContent>
            </Select>
        </div>
    )
}
