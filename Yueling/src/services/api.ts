// API服务
// 提供基础的HTTP请求方法
import { apiConfig } from '../config/api';

// API响应格式
export interface ApiResponse<T = any> {
    success: boolean;
    message?: string;
    data?: T;
    [key: string]: any;
}

export const api = {
    async post<T = any>(endpoint: string, data: any): Promise<ApiResponse<T>> {
        const response = await fetch(`${apiConfig.baseUrl}${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${await response.text()}`);
        }
        return response.json();
    },

    async get<T = any>(endpoint: string): Promise<ApiResponse<T>> {
        const response = await fetch(`${apiConfig.baseUrl}${endpoint}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${await response.text()}`);
        }
        return response.json();
    },

    async put<T = any>(endpoint: string, data: any): Promise<ApiResponse<T>> {
        const response = await fetch(`${apiConfig.baseUrl}${endpoint}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${await response.text()}`);
        }
        return response.json();
    },

    async upload<T = any>(endpoint: string, formData: FormData): Promise<ApiResponse<T>> {
        const response = await fetch(`${apiConfig.baseUrl}${endpoint}`, {
            method: 'POST',
            body: formData
        });
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${await response.text()}`);
        }
        return response.json();
    }
};