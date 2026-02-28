/// <reference types="vite/client" />

import 'axios'

declare module 'axios' {
  interface AxiosRequestConfig {
    /** When true, the global response interceptor will NOT show error toasts for this request. */
    _silentError?: boolean
  }
}
