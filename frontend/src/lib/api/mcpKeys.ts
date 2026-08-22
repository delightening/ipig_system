/**
 * MCP API Key 管理 API
 */
import api from './client'

export interface McpKey {
    id: string
    key_prefix: string
    name: string
    last_used_at: string | null
    created_at: string
    /** 權限範圍（read / write）；唯讀 key 僅 ['read'] */
    scopes: string[]
    /** 過期時間（null = 不過期，grandfather 既有 key） */
    expires_at: string | null
}

export interface CreateMcpKeyResponse extends McpKey {
    /** 完整金鑰，僅回傳一次 */
    full_key: string
}

export const mcpKeysApi = {
    list: async (): Promise<McpKey[]> => {
        const res = await api.get<McpKey[]>('/user/mcp-keys')
        return res.data
    },

    create: async (name: string, write: boolean): Promise<CreateMcpKeyResponse> => {
        const res = await api.post<CreateMcpKeyResponse>('/user/mcp-keys', { name, write })
        return res.data
    },

    revoke: async (id: string): Promise<void> => {
        await api.post(`/user/mcp-keys/${id}/revoke`)
    },
}
