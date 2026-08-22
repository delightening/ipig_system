const fs = require('fs');
const path = require('path');

const src = fs.readFileSync('d:/Coding/ipig_system/frontend/src/pages/protocols/ProtocolEditPage.tsx', 'utf8');
const lines = src.split('\n');
const outDir = 'd:/Coding/ipig_system/frontend/src/pages/protocols/protocol-edit';

const sections = [
    { name: 'Basic', key: 'basic', start: 1163, end: 1427 },
    { name: 'Purpose', key: 'purpose', start: 1429, end: 1624 },
    { name: 'Items', key: 'items', start: 1626, end: 1935 },
    { name: 'Design', key: 'design', start: 1937, end: 2843 },
    { name: 'Guidelines', key: 'guidelines', start: 2845, end: 2917 },
    { name: 'Surgery', key: 'surgery', start: 2918, end: 3335 },
    { name: 'Animals', key: 'animals', start: 3336, end: 3678 },
    { name: 'Personnel', key: 'personnel', start: 3679, end: 3848 },
    { name: 'Attachments', key: 'attachments', start: 3849, end: 3880 },
    { name: 'Signature', key: 'signature', start: 3881, end: 4234 },
];

sections.forEach(s => {
    const sectionLines = lines.slice(s.start - 1, s.end);
    const jsx = sectionLines.join('\n');

    const needsTextarea = jsx.includes('<Textarea') || jsx.includes('Textarea');
    const needsInput = jsx.includes('<Input');
    const needsLabel = jsx.includes('<Label');
    const needsSelect = jsx.includes('<Select') || jsx.includes('SelectTrigger');
    const needsCheckbox = jsx.includes('<Checkbox');
    const needsButton = jsx.includes('<Button');
    const needsBadge = jsx.includes('<Badge');
    const needsDatePicker = jsx.includes('<DatePicker');
    const needsFileUpload = jsx.includes('<FileUpload');
    const needsDialog = jsx.includes('<Dialog');
    const needsPlus = jsx.includes('<Plus');
    const needsUseAuthStore = jsx.includes('useAuthStore');

    let imports = [];

    // UI imports
    imports.push("import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'");
    if (needsInput || needsTextarea) {
        const parts = ['Input'];
        if (needsTextarea) parts.push('Textarea');
        imports.push("import { " + parts.join(', ') + " } from '@/components/ui/input'");
    }
    if (needsLabel) imports.push("import { Label } from '@/components/ui/label'");
    if (needsSelect) imports.push("import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'");
    if (needsCheckbox) imports.push("import { Checkbox } from '@/components/ui/checkbox'");
    if (needsButton) imports.push("import { Button } from '@/components/ui/button'");
    if (needsBadge) imports.push("import { Badge } from '@/components/ui/badge'");
    if (needsDatePicker) imports.push("import { DatePicker } from '@/components/ui/date-picker'");
    if (needsFileUpload) imports.push("import { FileUpload, FileInfo } from '@/components/ui/file-upload'");
    if (needsDialog) imports.push("import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog'");
    if (needsPlus) imports.push("import { Plus } from 'lucide-react'");
    if (needsUseAuthStore) imports.push("import { useAuthStore } from '@/stores/auth'");
    imports.push("import type { SectionProps } from './types'");

    // Extract inner JSX: skip first line {activeSection === 'xxx' && ( and last line )}
    const innerLines = sectionLines.slice(1, -1);

    // Dedent: original has ~12 spaces, reduce to 4
    const dedentedLines = innerLines.map(line => {
        if (line.startsWith('            ')) return '    ' + line.slice(12);
        if (line.startsWith('          ')) return '  ' + line.slice(10);
        return line;
    });

    const component = [
        `// Section ${s.name} 元件`,
        `// 自動從 ProtocolEditPage.tsx 提取`,
        '',
        ...imports,
        '',
        `export function Section${s.name}({ formData, updateWorkingContent, setFormData, t, isIACUCStaff }: SectionProps) {`,
        '',
        '  return (',
        ...dedentedLines,
        '  )',
        '}',
        '',
    ].join('\n');

    fs.writeFileSync(path.join(outDir, `Section${s.name}.tsx`), component, 'utf8');
    console.log(`Created Section${s.name}.tsx (${innerLines.length} lines)`);
});

console.log('\nAll sections extracted successfully!');
