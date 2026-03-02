<template>
  <div id="register-container" class="container" :class="{ active: isActive }">
    <div class="register-box">
      <div class="logo">
        <i class="fas fa-moon"></i>
        <h1>注册月灵账号</h1>
      </div>
      
      <form id="register-form" class="form" @submit.prevent="handleSubmit">
        <div class="form-group">
          <label for="register-username">用户名</label>
          <div class="input-wrapper">
            <i class="fas fa-user"></i>
            <input type="text" id="register-username" v-model="username" placeholder="请输入用户名" required>
          </div>
        </div>
        
        <div class="form-group">
          <label for="register-password">密码</label>
          <div class="input-wrapper">
            <i class="fas fa-lock"></i>
            <input type="password" id="register-password" v-model="password" placeholder="请输入密码" required>
            <i class="fas fa-eye-slash toggle-password" @click="togglePasswordVisibility('register')"></i>
          </div>
        </div>
        
        <div class="form-group">
          <label for="register-confirm-password">确认密码</label>
          <div class="input-wrapper">
            <i class="fas fa-lock"></i>
            <input type="password" id="register-confirm-password" v-model="confirmPassword" placeholder="请再次输入密码" required>
            <i class="fas fa-eye-slash toggle-password" @click="togglePasswordVisibility('register-confirm')"></i>
          </div>
        </div>
        
        <button type="submit" class="btn btn-primary">注册</button>
      </form>
      
      <div class="form-footer">
        <p>已有账号？ <a href="#" id="switch-to-login" @click.prevent="switchToLogin">立即登录</a></p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { RegisterData } from '../types'

export default defineComponent({
  name: 'Register',
  props: {
    isActive: {
      type: Boolean,
      default: false
    }
  },
  emits: ['register', 'switch-to-login'],
  setup(_, { emit }) {
    const username = ref('')
    const password = ref('')
    const confirmPassword = ref('')

    const handleSubmit = () => {
      const registerData: RegisterData = { username: username.value, password: password.value, confirmPassword: confirmPassword.value }
      emit('register', registerData)
    }

    const switchToLogin = () => {
      emit('switch-to-login')
    }

    const togglePasswordVisibility = (field: string) => {
      const input = document.getElementById(`${field}-password`) as HTMLInputElement
      if (input) {
        input.type = input.type === 'password' ? 'text' : 'password'
      }
    }

    return {
      username,
      password,
      confirmPassword,
      handleSubmit,
      switchToLogin,
      togglePasswordVisibility
    }
  }
})
</script>