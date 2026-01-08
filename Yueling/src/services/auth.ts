import { api } from './api'

export interface User {
    id: string
    username: string
}

export class AuthService {
    private currentUser: User | null = null

    async login(username: string, password: string): Promise<User> {
        const result = await api.post('/login', { username, password })
        if (result.success) {
            const user = { id: result.user_id, username: result.username }
            this.setCurrentUser(user)
            return user
        } else {
            throw new Error(result.message || '登录失败')
        }
    }

    async register(username: string, password: string, confirmPassword: string): Promise<void> {
        if (password !== confirmPassword) {
            throw new Error('两次输入的密码不一致')
        }
        const result = await api.post('/register', { username, password })
        if (!result.success) {
            throw new Error(result.message || '注册失败')
        }
    }

    logout() {
        this.currentUser = null
        localStorage.removeItem('currentUser')
    }

    getCurrentUser(): User | null {
        return this.currentUser
    }

    setCurrentUser(user: User) {
        this.currentUser = user
        localStorage.setItem('currentUser', JSON.stringify(user))
    }

    loadFromStorage(): User | null {
        const saved = localStorage.getItem('currentUser')
        if (saved) {
            try {
                const user = JSON.parse(saved)
                this.currentUser = user
                return user
            } catch (e) {
                console.error('Failed to parse stored user', e)
            }
        }
        return null
    }

    async checkUserExists(userId: string): Promise<boolean> {
        try {
            const result = await api.post('/user/exists', { user_id: userId })
            return result.success && result.exists
        } catch (error) {
            console.error('检查用户存在性失败:', error)
            return false
        }
    }
}

export const authService = new AuthService()