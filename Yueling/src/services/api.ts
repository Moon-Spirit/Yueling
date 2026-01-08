import { API_CONFIG } from '../config/api'

export const api = {
    async post(endpoint: string, data: any) {
        const response = await fetch(`${API_CONFIG.BASE_URL}${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        })
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${await response.text()}`)
        }
        return response.json()
    },

    async get(endpoint: string) {
        const response = await fetch(`${API_CONFIG.BASE_URL}${endpoint}`)
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${await response.text()}`)
        }
        return response.json()
    }
}