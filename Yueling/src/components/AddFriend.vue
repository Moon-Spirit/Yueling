<template>
  <div id="add-friend-container" class="container" :class="{ active: isActive }">
    <div class="register-box">
      <div class="logo">
        <i class="fas fa-user-plus"></i>
        <h1>添加好友</h1>
      </div>
      <form id="add-friend-form" class="form" @submit.prevent="handleSubmit">
        <div class="form-group">
          <label for="add-friend-username">好友用户名</label>
          <div class="input-wrapper">
            <i class="fas fa-user"></i>
            <input type="text" id="add-friend-username" v-model="username" placeholder="输入好友用户名" required>
          </div>
        </div>

        <div class="form-group">
          <label for="add-friend-display">显示名称（可选）</label>
          <div class="input-wrapper">
            <i class="fas fa-id-badge"></i>
            <input type="text" id="add-friend-display" v-model="displayName" placeholder="对好友显示的名称">
          </div>
        </div>

        <div class="form-group">
          <label for="add-friend-note">备注（可选）</label>
          <div class="input-wrapper">
            <i class="fas fa-sticky-note"></i>
            <input type="text" id="add-friend-note" v-model="note" placeholder="给好友写点备注">
          </div>
        </div>

        <div class="form-buttons">
          <button type="submit" class="btn btn-primary">发送好友请求</button>
          <button type="button" id="add-friend-cancel" class="btn btn-secondary" @click="cancel">取消</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { AddFriendData } from '../types'

export default defineComponent({
  name: 'AddFriend',
  props: {
    isActive: {
      type: Boolean,
      default: false
    }
  },
  emits: ['add-friend', 'cancel'],
  setup(_, { emit }) {
    const username = ref('')
    const displayName = ref('')
    const note = ref('')

    const handleSubmit = () => {
      const addFriendData: AddFriendData = { username: username.value, displayName: displayName.value, note: note.value }
      emit('add-friend', addFriendData)
    }

    const cancel = () => {
      emit('cancel')
    }

    return {
      username,
      displayName,
      note,
      handleSubmit,
      cancel
    }
  }
})
</script>