<template>
  <div class="voice-call" :class="{ active: isActive }">
    <!-- 音频元素 -->
    <audio ref="localAudio" autoplay muted></audio>
    <audio ref="remoteAudio" autoplay></audio>
    
    <div class="voice-call-header">
      <h3>语音通话</h3>
      <button class="close-btn" @click="close">
        <i class="fas fa-times"></i>
      </button>
    </div>
    
    <div class="voice-call-body">
      <div class="call-status">
        <div class="status-indicator" :class="statusClass"></div>
        <span class="status-text">{{ statusText }}</span>
      </div>
      
      <div class="call-info">
        <div class="caller-info">
          <div class="avatar">
            {{ remoteUserName?.charAt(0).toUpperCase() || '?' }}
          </div>
          <div class="caller-details">
            <h4>{{ remoteUserName || '未知用户' }}</h4>
            <p>{{ callDuration }}</p>
          </div>
        </div>
        
        <div class="network-quality" v-if="state.networkQuality">
          <h5>网络质量</h5>
          <div class="quality-metrics">
            <div class="metric">
              <span class="label">延迟:</span>
              <span class="value">{{ state.networkQuality.latency }}ms</span>
            </div>
            <div class="metric">
              <span class="label">丢包率:</span>
              <span class="value">{{ state.networkQuality.packetLoss.toFixed(1) }}%</span>
            </div>
            <div class="metric">
              <span class="label">抖动:</span>
              <span class="value">{{ state.networkQuality.jitter }}ms</span>
            </div>
            <div class="metric">
              <span class="label">连接模式:</span>
              <span class="value">{{ connectionModeText }}</span>
            </div>
          </div>
          
          <div class="connection-mode-switch" v-if="state.status === 'connected'">
            <label>手动切换连接模式:</label>
            <select v-model="selectedMode" @change="switchMode">
              <option value="p2p">P2P模式</option>
              <option value="server_relay">服务器中转模式</option>
            </select>
          </div>
        </div>
      </div>
    </div>
    
    <div class="voice-call-footer">
      <button class="end-call-btn" @click="endCall">
        <i class="fas fa-phone-slash"></i>
        结束通话
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted, onUnmounted } from 'vue'
import { voiceCallService, VoiceCallStatus, ConnectionMode, VoiceCallState } from '../services/voice'
import { User, Contact } from '../types'

export default defineComponent({
  name: 'VoiceCall',
  props: {
    isActive: {
      type: Boolean,
      default: false
    },
    currentUser: {
      type: Object as () => User | null,
      default: null
    },
    remoteUser: {
      type: Object as () => Contact | null,
      default: null
    }
  },
  emits: ['close', 'call-ended'],
  setup(props, { emit }) {
    const state = ref<VoiceCallState>(voiceCallService.getState())
    const callDuration = ref('00:00')
    const selectedMode = ref<ConnectionMode>(ConnectionMode.P2P)
    const durationInterval = ref<number | null>(null)
    const localAudio = ref<HTMLAudioElement | null>(null)
    const remoteAudio = ref<HTMLAudioElement | null>(null)

    const statusClass = computed(() => {
      switch (state.value.status) {
        case VoiceCallStatus.CONNECTING:
          return 'connecting'
        case VoiceCallStatus.CONNECTED:
          return 'connected'
        case VoiceCallStatus.FAILED:
          return 'failed'
        default:
          return 'idle'
      }
    })

    const statusText = computed(() => {
      switch (state.value.status) {
        case VoiceCallStatus.CONNECTING:
          return '正在连接...'
        case VoiceCallStatus.CONNECTED:
          return '通话中'
        case VoiceCallStatus.FAILED:
          return '连接失败'
        case VoiceCallStatus.DISCONNECTING:
          return '正在断开...'
        default:
          return '未连接'
      }
    })

    const connectionModeText = computed(() => {
      if (state.value.mode === ConnectionMode.P2P) {
        return 'P2P模式'
      } else if (state.value.mode === ConnectionMode.SERVER_RELAY) {
        return '服务器中转模式'
      }
      return '未知模式'
    })

    const remoteUserName = computed(() => {
      return props.remoteUser?.name || '未知用户'
    })

    const updateCallDuration = () => {
      if (state.value.startTime) {
        const duration = Math.floor((Date.now() - state.value.startTime) / 1000)
        const minutes = Math.floor(duration / 60)
        const seconds = duration % 60
        callDuration.value = `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
      }
    }

    const handleStateChange = (newState: VoiceCallState) => {
      state.value = newState
      selectedMode.value = newState.mode || ConnectionMode.P2P

      // 处理本地音频流
      if (newState.localStream && localAudio.value) {
        localAudio.value.srcObject = newState.localStream
      }

      // 处理远程音频流
      if (newState.remoteStream && remoteAudio.value) {
        remoteAudio.value.srcObject = newState.remoteStream
      }

      if (newState.status === VoiceCallStatus.CONNECTED) {
        // 开始计时
        updateCallDuration()
        durationInterval.value = window.setInterval(updateCallDuration, 1000)
      } else if (newState.status === VoiceCallStatus.IDLE) {
        // 停止计时
        if (durationInterval.value) {
          clearInterval(durationInterval.value)
          durationInterval.value = null
        }
        callDuration.value = '00:00'
        emit('call-ended')
      }
    }

    const endCall = () => {
      voiceCallService.endCall()
      emit('close')
    }

    const close = () => {
      emit('close')
    }

    const switchMode = async () => {
      try {
        await voiceCallService.switchConnectionMode(selectedMode.value)
      } catch (error) {
        console.error('Failed to switch connection mode:', error)
      }
    }

    onMounted(() => {
      voiceCallService.onStateChange(handleStateChange)
    })

    onUnmounted(() => {
      if (durationInterval.value) {
        clearInterval(durationInterval.value)
      }
    })

    return {
      state,
      callDuration,
      statusClass,
      statusText,
      connectionModeText,
      remoteUserName,
      selectedMode,
      localAudio,
      remoteAudio,
      endCall,
      close,
      switchMode
    }
  }
})
</script>

<style scoped>
.voice-call {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 90%;
  max-width: 400px;
  background: var(--bg-primary);
  border-radius: 20px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  z-index: 1000;
  opacity: 0;
  visibility: hidden;
  transition: all 0.3s ease;
}

.voice-call.active {
  opacity: 1;
  visibility: visible;
}

.voice-call-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid var(--border-color);
  border-radius: 20px 20px 0 0;
  background: var(--bg-secondary);
}

.voice-call-header h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 700;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 20px;
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.voice-call-body {
  padding: 30px 20px;
}

.call-status {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 30px;
  gap: 10px;
}

.status-indicator {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

.status-indicator.connecting {
  background: var(--warning-color);
}

.status-indicator.connected {
  background: var(--success-color);
}

.status-indicator.failed {
  background: var(--error-color);
}

.status-text {
  color: var(--text-secondary);
  font-size: 16px;
  font-weight: 500;
}

.caller-info {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 30px;
  gap: 20px;
}

.avatar {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 32px;
  font-weight: 700;
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.4);
}

.caller-details {
  text-align: center;
}

.caller-details h4 {
  margin: 0 0 8px 0;
  color: var(--text-primary);
  font-size: 20px;
  font-weight: 700;
}

.caller-details p {
  margin: 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.network-quality {
  background: var(--bg-secondary);
  padding: 20px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.network-quality h5 {
  margin: 0 0 15px 0;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.quality-metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 20px;
}

.metric {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-primary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.metric .label {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
}

.metric .value {
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 600;
}

.connection-mode-switch {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.connection-mode-switch label {
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
}

.connection-mode-switch select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.connection-mode-switch select:hover {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.1);
}

.voice-call-footer {
  padding: 20px;
  border-top: 1px solid var(--border-color);
  border-radius: 0 0 20px 20px;
  background: var(--bg-secondary);
  display: flex;
  justify-content: center;
}

.end-call-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 12px 30px;
  background: var(--error-color);
  color: white;
  border: none;
  border-radius: 25px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.end-call-btn:hover {
  background: #dc2626;
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(239, 68, 68, 0.4);
}

.end-call-btn:active {
  transform: translateY(0);
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

@media (max-width: 480px) {
  .voice-call {
    width: 95%;
  }

  .voice-call-header {
    padding: 16px;
  }

  .voice-call-body {
    padding: 20px 16px;
  }

  .caller-info {
    gap: 16px;
  }

  .avatar {
    width: 64px;
    height: 64px;
    font-size: 24px;
  }

  .caller-details h4 {
    font-size: 18px;
  }

  .network-quality {
    padding: 16px;
  }

  .quality-metrics {
    grid-template-columns: 1fr;
  }

  .voice-call-footer {
    padding: 16px;
  }

  .end-call-btn {
    padding: 10px 24px;
    font-size: 14px;
  }
}
</style>
