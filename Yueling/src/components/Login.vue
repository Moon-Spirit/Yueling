<template>
  <div id="login-container" class="container" :class="{ active: isActive }">
    <div class="login-box">
      <div class="logo">
        <i class="fas fa-moon"></i>
        <h1>月灵聊天</h1>
      </div>
      
      <form id="login-form" class="form" @submit.prevent="handleSubmit">
        <div class="form-group">
          <label for="login-username">用户名</label>
          <div class="input-wrapper">
            <i class="fas fa-user"></i>
            <input type="text" id="login-username" v-model="username" placeholder="请输入用户名" required>
          </div>
        </div>
        
        <div class="form-group">
          <label for="login-password">密码</label>
          <div class="input-wrapper">
            <i class="fas fa-lock"></i>
            <input type="password" id="login-password" v-model="password" placeholder="请输入密码" required>
            <i class="fas fa-eye-slash toggle-password" @click="togglePasswordVisibility"></i>
          </div>
        </div>
        
        <div class="form-options">
          <label class="checkbox">
            <input type="checkbox" id="remember-me" v-model="rememberMe">
            <span>记住密码</span>
          </label>
          <a href="#" class="forgot-password">忘记密码？</a>
        </div>
        
        <button type="submit" class="btn btn-primary">登录</button>
      </form>
      
      <div class="form-footer">
        <p>没有账号？ <a href="#" id="switch-to-register" @click.prevent="switchToRegister">立即注册</a></p>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { LoginData } from '../types'

export default defineComponent({
  name: 'Login',
  props: {
    isActive: {
      type: Boolean,
      default: false
    }
  },
  emits: ['login', 'switch-to-register'],
  setup(_, { emit }) {
    const username = ref('')
    const password = ref('')
    const rememberMe = ref(false)

    const handleSubmit = () => {
      const loginData: LoginData = { username: username.value, password: password.value, rememberMe: rememberMe.value }
      emit('login', loginData)
    }

    const switchToRegister = () => {
      emit('switch-to-register')
    }

    const togglePasswordVisibility = () => {
      const input = document.getElementById('login-password') as HTMLInputElement
      if (input) {
        input.type = input.type === 'password' ? 'text' : 'password'
      }
    }

    return {
      username,
      password,
      rememberMe,
      handleSubmit,
      switchToRegister,
      togglePasswordVisibility
    }
  }
})
</script>