import * as React from "react"
import { cn } from "@/lib/utils"
import { Upload, X, FileText, Image, Loader2 } from "lucide-react"
import { Button } from "./button"

export interface FileInfo {
  id: string
  file_name: string
  file_path: string
  file_size: number
  file_type?: string
  preview_url?: string
  created_at?: string
}

export interface FileUploadProps {
  value?: FileInfo[]
  onChange?: (files: FileInfo[]) => void
  onUpload?: (file: File) => Promise<FileInfo>
  accept?: string
  multiple?: boolean
  maxSize?: number // in MB
  maxFiles?: number
  disabled?: boolean
  className?: string
  placeholder?: string
  showPreview?: boolean
}

const FileUpload = React.forwardRef<HTMLDivElement, FileUploadProps>(
  ({
    value = [],
    onChange,
    onUpload,
    accept = "*/*",
    multiple = true,
    maxSize = 10,
    maxFiles = 10,
    disabled = false,
    className,
    placeholder = "拖曳檔案到此處，或點擊選擇檔案",
    showPreview = true,
  }, ref) => {
    const [isDragging, setIsDragging] = React.useState(false)
    const [uploading, setUploading] = React.useState(false)
    const [error, setError] = React.useState<string | null>(null)
    const inputRef = React.useRef<HTMLInputElement>(null)

    const isImage = (fileName: string) => {
      return /\.(jpg|jpeg|png|gif|webp|bmp)$/i.test(fileName)
    }

    const formatFileSize = (bytes: number) => {
      if (bytes < 1024) return `${bytes} B`
      if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
    }

    const handleFiles = async (files: FileList | null) => {
      if (!files || files.length === 0 || disabled) return

      setError(null)
      const fileArray = Array.from(files)

      // Validate file count
      if (value.length + fileArray.length > maxFiles) {
        setError(`最多只能上傳 ${maxFiles} 個檔案`)
        return
      }

      // Validate file sizes
      for (const file of fileArray) {
        if (file.size > maxSize * 1024 * 1024) {
          setError(`檔案 ${file.name} 超過 ${maxSize}MB 限制`)
          return
        }
      }

      if (onUpload) {
        setUploading(true)
        try {
          const uploadedFiles: FileInfo[] = []
          for (const file of fileArray) {
            const uploadedFile = await onUpload(file)
            uploadedFiles.push(uploadedFile)
          }
          onChange?.([...value, ...uploadedFiles])
        } catch (err) {
          setError('上傳失敗，請重試')
          console.error('Upload error:', err)
        } finally {
          setUploading(false)
        }
      } else {
        // Local preview without upload
        const newFiles: FileInfo[] = fileArray.map((file) => ({
          id: `local-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          file_name: file.name,
          file_path: '',
          file_size: file.size,
          file_type: file.type,
          preview_url: isImage(file.name) ? URL.createObjectURL(file) : undefined,
        }))
        onChange?.([...value, ...newFiles])
      }
    }

    const handleDragOver = (e: React.DragEvent) => {
      e.preventDefault()
      if (!disabled) setIsDragging(true)
    }

    const handleDragLeave = (e: React.DragEvent) => {
      e.preventDefault()
      setIsDragging(false)
    }

    const handleDrop = (e: React.DragEvent) => {
      e.preventDefault()
      setIsDragging(false)
      handleFiles(e.dataTransfer.files)
    }

    const handleRemove = (fileId: string) => {
      if (disabled) return
      const fileToRemove = value.find(f => f.id === fileId)
      if (fileToRemove?.preview_url?.startsWith('blob:')) {
        URL.revokeObjectURL(fileToRemove.preview_url)
      }
      onChange?.(value.filter((f) => f.id !== fileId))
    }

    const handleClick = () => {
      if (!disabled) inputRef.current?.click()
    }

    return (
      <div ref={ref} className={cn("space-y-3", className)}>
        {/* Drop Zone */}
        <div
          onClick={handleClick}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          className={cn(
            "relative flex flex-col items-center justify-center rounded-lg border-2 border-dashed p-6 transition-colors cursor-pointer",
            isDragging
              ? "border-purple-500 bg-purple-50"
              : "border-slate-300 hover:border-slate-400 hover:bg-slate-50",
            disabled && "cursor-not-allowed opacity-50",
            uploading && "pointer-events-none"
          )}
        >
          <input
            ref={inputRef}
            type="file"
            accept={accept}
            multiple={multiple}
            onChange={(e) => handleFiles(e.target.files)}
            className="hidden"
            disabled={disabled}
          />
          
          {uploading ? (
            <>
              <Loader2 className="h-8 w-8 animate-spin text-purple-500 mb-2" />
              <p className="text-sm text-slate-600">上傳中...</p>
            </>
          ) : (
            <>
              <Upload className="h-8 w-8 text-slate-400 mb-2" />
              <p className="text-sm text-slate-600 text-center">{placeholder}</p>
              <p className="text-xs text-slate-400 mt-1">
                最大 {maxSize}MB，最多 {maxFiles} 個檔案
              </p>
            </>
          )}
        </div>

        {/* Error Message */}
        {error && (
          <p className="text-sm text-red-500">{error}</p>
        )}

        {/* File List */}
        {value.length > 0 && (
          <div className="space-y-2">
            {value.map((file) => (
              <div
                key={file.id}
                className="flex items-center gap-3 p-2 rounded-lg border bg-slate-50"
              >
                {/* Preview or Icon */}
                {showPreview && file.preview_url ? (
                  <img
                    src={file.preview_url}
                    alt={file.file_name}
                    className="h-10 w-10 rounded object-cover"
                  />
                ) : isImage(file.file_name) ? (
                  <div className="h-10 w-10 rounded bg-blue-100 flex items-center justify-center">
                    <Image className="h-5 w-5 text-blue-600" />
                  </div>
                ) : (
                  <div className="h-10 w-10 rounded bg-slate-200 flex items-center justify-center">
                    <FileText className="h-5 w-5 text-slate-600" />
                  </div>
                )}

                {/* File Info */}
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{file.file_name}</p>
                  <p className="text-xs text-slate-500">{formatFileSize(file.file_size)}</p>
                </div>

                {/* Remove Button */}
                {!disabled && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 shrink-0"
                    onClick={(e) => {
                      e.stopPropagation()
                      handleRemove(file.id)
                    }}
                  >
                    <X className="h-4 w-4" />
                  </Button>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    )
  }
)
FileUpload.displayName = "FileUpload"

export { FileUpload }
