/**
 * R20-3: 驗證結果面板
 *
 * 顯示 Level 1 規則引擎的驗證結果：
 * - errors（紅色）：必須修正
 * - warnings（黃色）：建議改善
 * - passed（綠色，可摺疊）：已通過
 */
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { AlertCircle, AlertTriangle, CheckCircle2, ChevronDown, ChevronRight } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import type { ValidationResult } from '@/types/aiReview'

interface ValidationPanelProps {
    result: ValidationResult
    onDismiss?: () => void
    onIgnoreAndSubmit?: () => void
    hasErrors: boolean
}

export function ValidationPanel({
    result,
    onDismiss,
    onIgnoreAndSubmit,
    hasErrors,
}: ValidationPanelProps) {
    const { t } = useTranslation()
    const [showPassed, setShowPassed] = useState(false)

    return (
        <Card className="border-2 border-border">
            <CardHeader className="pb-3">
                <CardTitle className="text-lg flex items-center gap-2">
                    <AlertCircle className="h-5 w-5" />
                    {t('protocolComponents.validation.title')}
                </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* Errors */}
                {result.errors.length > 0 && (
                    <div className="space-y-2">
                        <h4 className="text-sm font-semibold flex items-center gap-1.5 text-status-error-text">
                            <AlertCircle className="h-4 w-4" />
                            {t('protocolComponents.shared.mustFixCount', { count: result.errors.length })}
                        </h4>
                        <ul className="space-y-2 ml-5">
                            {result.errors.map((issue) => (
                                <li key={issue.code} className="text-sm">
                                    <div className="font-medium text-status-error-text">
                                        [{issue.category}] {issue.message}
                                    </div>
                                    <div className="text-muted-foreground mt-0.5">
                                        {issue.suggestion}
                                    </div>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                {/* Warnings */}
                {result.warnings.length > 0 && (
                    <div className="space-y-2">
                        <h4 className="text-sm font-semibold flex items-center gap-1.5 text-status-warning-text">
                            <AlertTriangle className="h-4 w-4" />
                            {t('protocolComponents.shared.suggestionsCount', { count: result.warnings.length })}
                        </h4>
                        <ul className="space-y-2 ml-5">
                            {result.warnings.map((issue) => (
                                <li key={issue.code} className="text-sm">
                                    <div className="font-medium text-status-warning-text">
                                        [{issue.category}] {issue.message}
                                    </div>
                                    <div className="text-muted-foreground mt-0.5">
                                        {issue.suggestion}
                                    </div>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                {/* Passed (collapsible) */}
                {result.passed.length > 0 && (
                    <div className="space-y-1">
                        <button
                            onClick={() => setShowPassed(!showPassed)}
                            className="text-sm font-semibold flex items-center gap-1.5 text-status-success-text hover:underline"
                        >
                            {showPassed ? (
                                <ChevronDown className="h-4 w-4" />
                            ) : (
                                <ChevronRight className="h-4 w-4" />
                            )}
                            <CheckCircle2 className="h-4 w-4" />
                            {t('protocolComponents.shared.passedCount', { count: result.passed.length })}
                        </button>
                        {showPassed && (
                            <div className="ml-5 text-sm text-muted-foreground">
                                {result.passed.join(t('protocolComponents.shared.listSeparator'))}
                            </div>
                        )}
                    </div>
                )}

                {/* Actions */}
                <div className="flex gap-2 pt-2 border-t">
                    {onDismiss && (
                        <Button variant="outline" size="sm" onClick={onDismiss}>
                            {t('protocolComponents.validation.backToEdit')}
                        </Button>
                    )}
                    {!hasErrors && onIgnoreAndSubmit && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={onIgnoreAndSubmit}
                        >
                            {t('protocolComponents.validation.ignoreAndSubmit')}
                        </Button>
                    )}
                </div>
            </CardContent>
        </Card>
    )
}
