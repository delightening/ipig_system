import type { Meta, StoryObj } from '@storybook/react-vite'
import { PdfExportButtons } from './PdfExportButtons'

const meta = {
    title: 'PDF Export/PdfExportButtons',
    component: PdfExportButtons,
    tags: ['autodocs'],
    parameters: { layout: 'centered' },
    args: {
        baseUrl: '/api/v1/protocols/demo-123',
    },
} satisfies Meta<typeof PdfExportButtons>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const PreviewOnly: Story = {
    name: '只顯示預覽',
    args: {
        showDownloadDocx: false,
        showDownloadPdf: false,
    },
}

export const DownloadOnly: Story = {
    name: '只顯示下載按鈕',
    args: {
        showPreview: false,
    },
}

export const CustomPreviewTitle: Story = {
    name: '自訂預覽標題',
    args: {
        previewTitle: '計畫書 PDF 預覽',
    },
}
