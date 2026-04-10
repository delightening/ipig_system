import * as React from "react"
import { cn } from "@/lib/utils"
import { logger } from "@/lib/logger"
import { Upload, X, FileText, Image, Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
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
  hint?: string
  hideHint?: boolean
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
    placeholder,
    showPreview = true,
    hint,
    hideHint = false,
  }, ref) => {
    const { t } = useTranslation()
    const [isDragging, setIsDragging] = React.useState(false)
    const [uploading, setUploading] = React.useState(false)
    const [error, setError] = React.useState<string | null>(null)
    const inputRef = React.useRef<HTMLInputElement>(null)

    // L-01: 元件卸載時清除所有 blob URL，防止記憶體洩漏
    React.useEffect(() => {
      return () => {
        value.forEach((f) => {
          if (f.preview_url?.startsWith('blob:')) {
            URL.revokeObjectURL(f.preview_url)
          }
        })
      }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    const isImage = (fileName: string) => {
      return /\.(jpg|jpeg|png|gif|webp|bmp)$/i.test(fileName)
    }

    /** 允許的 MIME type 白名單 */
    const ALLOWED_MIME_TYPES: Record<string, boolean> = {
      // 圖片
      'image/jpeg': true,
      'image/png': true,
      'image/gif': true,
      'image/webp': true,
      'image/bmp': true,
      'image/svg+xml': true,
      // 文件
      'application/pdf': true,
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': true, // .xlsx
      'application/vnd.ms-excel': true, // .xls
      'text/csv': true,
      'application/msword': true, // .doc
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document': true, // .docx
      'application/vnd.openxmlformats-officedocument.presentationml.presentation': true, // .pptx
      'application/vnd.ms-powerpoint': true, // .ppt
      'text/plain': true,
      'application/zip': true,
      'application/x-zip-compressed': true,
    }

    /** 允許的副檔名白名單（MIME type 為空時降級使用） */
    const ALLOWED_EXTENSIONS = /\.(jpg|jpeg|png|gif|webp|bmp|svg|pdf|xlsx|xls|csv|doc|docx|pptx|ppt|txt|zip)$/i

    /** 驗證檔案 MIME type 是否在白名單內 */
    const validateMimeType = (file: File): string | null => {
      // 某些瀏覽器或系統可能回傳空字串 MIME type，此時降級為只檢查副檔名
      if (!file.type) {
        if (!ALLOWED_EXTENSIONS.test(file.name)) {
          return t('common.fileUpload.errorMimeType', {
            fileName: file.name,
            defaultValue: `檔案 "${file.name}" 的類型不在允許範圍內`,
          })
        }
        return null
      }
      // 允許 image/* 通配
      if (file.type.startsWith('image/') || ALLOWED_MIME_TYPES[file.type]) {
        return null
      }
      return t('common.fileUpload.errorMimeType', {
        fileName: file.name,
        defaultValue: `檔案 "${file.name}" 的類型 (${file.type}) 不在允許範圍內`,
      })
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
        setError(t('common.fileUpload.errorMaxFiles', { maxFiles }))
        return
      }

      // Validate MIME types
      for (const file of fileArray) {
        const mimeError = validateMimeType(file)
        if (mimeError) {
          setError(mimeError)
          return
        }
      }

      // Validate file sizes
      for (const file of fileArray) {
        if (file.size > maxSize * 1024 * 1024) {
          setError(t('common.fileUpload.errorMaxSize', { fileName: file.name, maxSize }))
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
          setError(t('common.fileUpload.errorUploadFailed'))
          logger.error('Upload error:', err)
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
              ? "border-purple-500 bg-status-purple-bg"
              : "border-border hover:border-slate-400 hover:bg-muted",
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
            aria-label={placeholder || t('common.fileUpload.placeholder')}
            title={placeholder || t('common.fileUpload.placeholder')}
          />

          {uploading ? (
            <>
              <Loader2 className="h-8 w-8 animate-spin text-status-purple-solid mb-2" />
              <p className="text-sm text-muted-foreground">{t('common.fileUpload.uploading')}</p>
            </>
          ) : (
            <>
              <Upload className="h-8 w-8 text-muted-foreground mb-2" />
              <p className="text-sm text-muted-foreground text-center">
                {placeholder || t('common.fileUpload.placeholder')}
              </p>
              {!hideHint && (
                <p className="text-xs text-muted-foreground mt-1">
                  {hint || t('common.fileUpload.hint', { maxSize, maxFiles })}
                </p>
              )}
            </>
          )}
        </div>

        {/* Error Message */}
        {error && (
          <p className="text-sm text-status-error-solid">{error}</p>
        )}

        {/* File List */}
        {value.length > 0 && (
          <div className="space-y-2">
            {value.map((file) => (
              <div
                key={file.id}
                className="flex items-center gap-3 p-2 rounded-lg border bg-muted"
              >
                {/* Preview or Icon */}
                {showPreview && file.preview_url ? (
                  <img
                    src={file.preview_url}
                    alt={file.file_name}
                    className="h-10 w-10 rounded object-cover"
                  />
                ) : isImage(file.file_name) ? (
                  <div className="h-10 w-10 rounded bg-status-info-bg flex items-center justify-center">
                    <Image className="h-5 w-5 text-status-info-text" />
                  </div>
                ) : (
                  <div className="h-10 w-10 rounded bg-muted flex items-center justify-center">
                    <FileText className="h-5 w-5 text-muted-foreground" />
                  </div>
                )}

                {/* File Info */}
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{file.file_name}</p>
                  <p className="text-xs text-muted-foreground">{formatFileSize(file.file_size)}</p>
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
