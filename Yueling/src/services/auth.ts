import { api } from './api'
import { User } from '../types'

export class AuthService {
    private currentUser: User | null = null

    async login(username: string, password: string): Promise<User> {
        const result = await api.post('/login', { username, password })
        if (result.success) {
            // 先创建基本用户对象
            const user: User = { id: result.user_id, username: result.username, avatar_url: '' }
            this.setCurrentUser(user)
            // 获取用户详细信息（包括头像）
            try {
                const userInfo = await this.getUserInfo(result.user_id)
                user.avatar_url = userInfo.avatar_url || ''
                this.setCurrentUser(user)
            } catch (error) {
                console.error('获取用户头像失败:', error)
            }
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

    async getUserInfo(userId: string): Promise<{ avatar_url?: string }> {
        try {
            const result = await api.get(`/user/${userId}`)
            if (result.success && result.user) {
                return result.user
            }
            return {}
        } catch (error) {
            console.error('获取用户信息失败:', error)
            return {}
        }
    }

    async uploadAvatar(userId: string, file: File): Promise<string> {
        const formData = new FormData()
        formData.append('avatar', file)
        
        try {
            const result = await api.upload(`/user/${userId}/avatar`, formData)
            if (result.success && result.avatar_url) {
                // 更新当前用户的头像URL
                const currentUser = this.getCurrentUser()
                if (currentUser) {
                    currentUser.avatar_url = result.avatar_url
                    this.setCurrentUser(currentUser)
                }
                return result.avatar_url
            }
            throw new Error(result.message || '上传头像失败')
        } catch (error) {
            console.error('上传头像失败:', error)
            throw error
        }
    }

    async updateUserInfo(userId: string, username: string, email: string): Promise<void> {
        try {
            const result = await api.put(`/user/${userId}`, { username, email })
            if (!result.success) {
                throw new Error(result.message || '更新用户信息失败')
            }
            // 更新当前用户的用户名
            const currentUser = this.getCurrentUser()
            if (currentUser) {
                currentUser.username = username
                this.setCurrentUser(currentUser)
            }
        } catch (error) {
            console.error('更新用户信息失败:', error)
            throw error
        }
    }
}

export const authService = new AuthService()