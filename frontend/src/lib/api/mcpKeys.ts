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

    create: async (name: string): Promise<CreateMcpKeyResponse> => {
        const res = await api.post<CreateMcpKeyResponse>('/user/mcp-keys', { name })
        return res.data
    },

    revoke: async (id: string): Promise<void> => {
        await api.post(`/user/mcp-keys/${id}/revoke`)
    },
}
