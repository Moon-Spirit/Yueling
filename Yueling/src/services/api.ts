import { API_CONFIG } from '../config/api'
import { ApiResponse } from '../types'

export class ApiService {
    private baseUrl: string

    constructor() {
        this.baseUrl = API_CONFIG.BASE_URL
    }

    private async handleResponse<T>(response: Response): Promise<ApiResponse<T>> {
        const data = await response.json()
        if (!response.ok) {
            throw new Error(data.message || `HTTP ${response.status}`)
        }
        return data
    }

    async post<T>(endpoint: string, data: any): Promise<ApiResponse<T>> {
        const response = await fetch(`${this.baseUrl}${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        })
        return this.handleResponse<T>(response)
    }

    async get<T>(endpoint: string): Promise<ApiResponse<T>> {
        const response = await fetch(`${this.baseUrl}${endpoint}`)
        return this.handleResponse<T>(response)
    }

    async put<T>(endpoint: string, data: any): Promise<ApiResponse<T>> {
        const response = await fetch(`${this.baseUrl}${endpoint}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        })
        return this.handleResponse<T>(response)
    }

    async upload<T>(endpoint: string, formData: FormData): Promise<ApiResponse<T>> {
        const response = await fetch(`${this.baseUrl}${endpoint}`, {
            method: 'POST',
            body: formData
        })
        return this.handleResponse<T>(response)
    }
}

export const api = new ApiService()