<template>
  <div id="profile-container" class="container" :class="{ active: isActive }">
    <div class="register-box" style="max-width: 600px;">
      <div class="logo">
        <i class="fas fa-user"></i>
        <h1>个人主页</h1>
      </div>
      <div class="form">
        <div class="form-group" style="text-align: center;">
          <div class="avatar-container" style="display: inline-block; margin-bottom: 20px;">
            <div class="avatar" @click="showAvatarUpload" style="width: 120px; height: 120px;">
              <img v-if="currentUser?.avatar_url" :src="`${API_CONFIG.BASE_URL}${currentUser.avatar_url}`" :alt="currentUser.username" class="avatar-img" style="width: 100%; height: 100%;">
              <i v-else class="fas fa-user-circle avatar-icon" style="font-size: 120px;"></i>
              <input type="file" ref="avatarInput" style="display: none" accept="image/*" @change="handleAvatarChange">
            </div>
            <span class="status online">在线</span>
          </div>
        </div>

        <div class="form-group">
          <label for="profile-username">用户名</label>
          <div class="input-wrapper">
            <i class="fas fa-user"></i>
            <input type="text" id="profile-username" v-model="username" placeholder="请输入用户名">
          </div>
        </div>

        <div class="form-group">
          <label for="profile-bio">个人简介</label>
          <div class="input-wrapper">
            <i class="fas fa-info-circle"></i>
            <textarea id="profile-bio" v-model="bio" placeholder="介绍一下自己..." rows="3"></textarea>
          </div>
        </div>

        <div class="form-group">
          <label for="profile-email">邮箱</label>
          <div class="input-wrapper">
            <i class="fas fa-envelope"></i>
            <input type="email" id="profile-email" v-model="email" placeholder="请输入邮箱">
          </div>
        </div>

        <div class="form-buttons">
          <button type="button" class="btn btn-primary" @click="saveProfile">保存修改</button>
          <button type="button" class="btn btn-secondary" @click="cancel">返回</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, PropType } from 'vue'
import { API_CONFIG } from '../config/api'
import { User, ProfileData } from '../types'

export default defineComponent({
  name: 'Profile',
  props: {
    isActive: {
      type: Boolean,
      default: false
    },
    currentUser: {
      type: Object as PropType<User | null>,
      default: null
    }
  },
  emits: ['save-profile', 'avatar-change', 'cancel'],
  setup(props, { emit }) {
    const username = ref(props.currentUser?.username || '')
    const bio = ref('')
    const email = ref('')
    const avatarInput = ref<HTMLInputElement | null>(null)

    const showAvatarUpload = () => {
      if (avatarInput.value) {
        avatarInput.value.click()
      }
    }

    const handleAvatarChange = (event: Event) => {
      const input = event.target as HTMLInputElement
      if (input.files && input.files.length > 0 && props.currentUser) {
        const file = input.files[0]
        emit('avatar-change', file)
        input.value = ''
      }
    }

    const saveProfile = () => {
      const profileData: ProfileData = { username: username.value, bio: bio.value, email: email.value }
      emit('save-profile', profileData)
    }

    const cancel = () => {
      emit('cancel')
    }

    return {
      username,
      bio,
      email,
      avatarInput,
      API_CONFIG,
      showAvatarUpload,
      handleAvatarChange,
      saveProfile,
      cancel
    }
  }
})
</script>