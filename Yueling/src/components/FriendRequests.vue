<template>
  <div id="friend-requests-container" class="container" :class="{ active: isActive }">
    <div class="register-box">
      <div class="logo">
        <i class="fas fa-user-check"></i>
        <h1>好友请求</h1>
      </div>
      <div class="form">
        <div class="form-group">
          <label>收到的好友请求</label>
          <div class="input-wrapper">
            <div id="friend-requests-list" style="width:100%;">
              <div v-for="request in friendRequests" :key="request.id" class="friend-request-item">
                <div style="display:flex;justify-content:space-between;align-items:center;">
                  <div>
                    <div><strong>{{ request.from_username || request.from_user_id || '未知用户' }}</strong></div>
                    <div style="font-size:12px;color:#666;margin-top:4px;">请求时间：{{ formatRequestTime(request.created_at) }}</div>
                  </div>
                  <div style="display:flex;gap:12px;">
                    <button class="btn btn-accept" @click="respondToRequest(request.id, 'accepted')">接受</button>
                    <button class="btn btn-reject" @click="respondToRequest(request.id, 'rejected')">拒绝</button>
                  </div>
                </div>
              </div>
              <div v-if="friendRequests.length === 0" style="text-align:center;color:var(--text-secondary);padding:24px;background:var(--bg-light);border-radius:12px;margin-top:12px;">没有新的好友请求。</div>
            </div>
          </div>
        </div>

        <div class="form-buttons">
          <button type="button" id="friend-requests-cancel" class="btn btn-primary" @click="cancel">返回</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import { FriendRequest, FriendRequestResponse } from '../types'

export default defineComponent({
  name: 'FriendRequests',
  props: {
    isActive: {
      type: Boolean,
      default: false
    },
    friendRequests: {
      type: Array as PropType<FriendRequest[]>,
      default: () => []
    }
  },
  emits: ['respond-request', 'cancel'],
  setup(_, { emit }) {
    const respondToRequest = (requestId: string, response: 'accepted' | 'rejected') => {
      const responseData: FriendRequestResponse = { requestId, response }
      emit('respond-request', responseData)
    }

    const cancel = () => {
      emit('cancel')
    }

    const formatRequestTime = (timestamp: number) => {
      return new Date(timestamp * 1000).toLocaleString('zh-CN')
    }

    return {
      respondToRequest,
      cancel,
      formatRequestTime
    }
  }
})
</script>