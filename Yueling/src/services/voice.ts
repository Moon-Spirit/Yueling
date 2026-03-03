import { websocketService } from './websocket'

// 语音通话状态枚举
export enum VoiceCallStatus {
  IDLE = 'idle',
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  DISCONNECTING = 'disconnecting',
  FAILED = 'failed'
}

// 连接模式枚举
export enum ConnectionMode {
  P2P = 'p2p',
  SERVER_RELAY = 'server_relay'
}

// 网络质量评估结果
export interface NetworkQuality {
  latency: number; // 延迟（毫秒）
  packetLoss: number; // 丢包率（百分比）
  jitter: number; // 抖动（毫秒）
  recommendedMode: ConnectionMode; // 推荐的连接模式
}

// 语音通话配置
export interface VoiceCallConfig {
  stunServers: RTCIceServer[];
  turnServers: RTCIceServer[];
  opusBitrate: number; // OPUS编码比特率
  sampleRate: number; // 采样率
  maxLatency: number; // 最大延迟阈值
  maxPacketLoss: number; // 最大丢包率阈值
}

// 语音通话状态
export interface VoiceCallState {
  status: VoiceCallStatus;
  mode: ConnectionMode | null;
  networkQuality: NetworkQuality | null;
  localStream: MediaStream | null;
  remoteStream: MediaStream | null;
  peerConnection: RTCPeerConnection | null;
  callId: string | null;
  userId: string | null;
  remoteUserId: string | null;
  startTime: number | null;
  endTime: number | null;
}

export class VoiceCallService {
  private state: VoiceCallState = {
    status: VoiceCallStatus.IDLE,
    mode: null,
    networkQuality: null,
    localStream: null,
    remoteStream: null,
    peerConnection: null,
    callId: null,
    userId: null,
    remoteUserId: null,
    startTime: null,
    endTime: null
  };

  private config: VoiceCallConfig = {
    stunServers: [
      { urls: 'stun:stun.l.google.com:19302' },
      { urls: 'stun:stun1.l.google.com:19302' }
    ],
    turnServers: [
      { 
        urls: 'turn:turn.example.com:443',
        username: 'username',
        credential: 'password'
      }
    ],
    opusBitrate: 64000, // 64kbps (降低比特率减少卡顿)
    sampleRate: 48000, // 48kHz
    maxLatency: 400, // 增加最大延迟阈值
    maxPacketLoss: 8 // 增加最大丢包率阈值
  };

  private qualityCheckInterval: number | null = null;
  private connectionStateTimer: number | null = null;
  private stateChangeCallbacks: ((state: VoiceCallState) => void)[] = [];

  constructor() {
    this.setupWebSocketListeners();
  }

  private setupWebSocketListeners() {
    // 确保WebSocket服务已经连接
    websocketService.connect().then(() => {
      console.log('WebSocket connected for voice calls');
      // 注册语音通话相关的消息监听器
      const handleVoiceCallOffer = (data: any) => {
        console.log('收到语音通话邀请:', data);
        this.handleVoiceCallOffer(data);
      };
      const handleVoiceCallAnswer = (data: any) => {
        console.log('收到语音通话应答:', data);
        this.handleVoiceCallAnswer(data);
      };
      const handleIceCandidate = (data: any) => {
        console.log('收到ICE候选:', data);
        this.handleIceCandidate(data);
      };
      const handleVoiceCallEnd = (data: any) => {
        console.log('收到语音通话结束:', data);
        this.handleVoiceCallEnd(data);
      };
      websocketService.on('voice_call_offer', handleVoiceCallOffer);
      websocketService.on('voice_call_answer', handleVoiceCallAnswer);
      websocketService.on('ice_candidate', handleIceCandidate);
      websocketService.on('voice_call_end', handleVoiceCallEnd);
    }).catch(err => {
      console.error('Failed to connect WebSocket for voice calls:', err);
    });
  }

  private async getLocalStream(): Promise<MediaStream> {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
          sampleRate: 48000,
          channelCount: 1
        }
      });
      this.state.localStream = stream;
      return stream;
    } catch (error) {
      console.error('Failed to get local stream:', error);
      throw new Error('Failed to access microphone');
    }
  }

  private createPeerConnection(): RTCPeerConnection {
    const configuration = {
      iceServers: [...this.config.stunServers, ...this.config.turnServers]
    };

    const peerConnection = new RTCPeerConnection(configuration);

    // 添加本地流
    if (this.state.localStream) {
      this.state.localStream.getAudioTracks().forEach(track => {
        peerConnection.addTrack(track, this.state.localStream!);
      });
    }

    // 处理远程流
    peerConnection.ontrack = (event) => {
      if (!this.state.remoteStream) {
        this.state.remoteStream = new MediaStream();
      }
      event.streams[0].getAudioTracks().forEach(track => {
        this.state.remoteStream!.addTrack(track);
      });
      // 如果连接已经建立，更新状态为CONNECTED
      if (peerConnection.connectionState === 'connected' && this.state.status !== VoiceCallStatus.CONNECTED) {
        this.state.status = VoiceCallStatus.CONNECTED;
        this.state.startTime = Date.now();
      }
      this.updateState();
    };

    // 处理ICE候选
    peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        websocketService.send({
          type: 'ice_candidate',
          call_id: this.state.callId,
          candidate: event.candidate,
          remote_user_id: this.state.remoteUserId
        });
      }
    };

    // 处理连接状态变化
    peerConnection.onconnectionstatechange = () => {
      console.log('Peer connection state:', peerConnection.connectionState);
      if (peerConnection.connectionState === 'connected') {
        // 清除可能存在的失败定时器
        if (this.connectionStateTimer) {
          clearTimeout(this.connectionStateTimer);
          this.connectionStateTimer = null;
        }
        this.state.status = VoiceCallStatus.CONNECTED;
        this.state.startTime = Date.now();
        this.updateState();
      } else if (peerConnection.connectionState === 'disconnected' || peerConnection.connectionState === 'failed') {
        // 只有在真正断开连接时才标记为失败
        // 有时候连接状态会短暂变为disconnected但很快恢复
        console.log('Connection state changed to:', peerConnection.connectionState);
        // 清除之前的定时器
        if (this.connectionStateTimer) {
          clearTimeout(this.connectionStateTimer);
        }
        // 不立即标记为失败，而是尝试重新建立连接
        this.connectionStateTimer = window.setTimeout(() => {
          if (this.state.peerConnection && (this.state.peerConnection.connectionState === 'disconnected' || this.state.peerConnection.connectionState === 'failed')) {
            this.state.status = VoiceCallStatus.FAILED;
            this.updateState();
            this.switchToServerRelay();
          }
        }, 3000); // 增加延迟到3秒，减少误报
      }
    };

    return peerConnection;
  }

  private async handleVoiceCallOffer(data: any) {
    try {
      this.state.callId = data.call_id;
      this.state.remoteUserId = data.sender_id;
      this.state.status = VoiceCallStatus.CONNECTING;
      this.updateState();

      // 获取本地流
      await this.getLocalStream();

      // 创建对等连接
      this.state.peerConnection = this.createPeerConnection();
      this.state.mode = ConnectionMode.P2P;

      // 设置远程描述
      await this.state.peerConnection.setRemoteDescription(new RTCSessionDescription(data.offer));

      // 创建应答
      const answer = await this.state.peerConnection.createAnswer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: false
      });
      await this.state.peerConnection.setLocalDescription(answer);

      // 发送应答
      websocketService.send({
        type: 'voice_call_answer',
        call_id: this.state.callId,
        answer: answer,
        remote_user_id: this.state.remoteUserId
      });

      // 开始网络质量监测
      this.startQualityMonitoring();
    } catch (error) {
      console.error('Failed to handle voice call offer:', error);
      this.state.status = VoiceCallStatus.FAILED;
      this.updateState();
    }
  }

  private async handleVoiceCallAnswer(data: any) {
    try {
      if (this.state.peerConnection) {
        await this.state.peerConnection.setRemoteDescription(new RTCSessionDescription(data.answer));
        // 检查连接状态，如果已经连接则更新状态
        if (this.state.peerConnection.connectionState === 'connected') {
          this.state.status = VoiceCallStatus.CONNECTED;
          this.state.startTime = Date.now();
          this.updateState();
        }
      }
    } catch (error) {
      console.error('Failed to handle voice call answer:', error);
      this.state.status = VoiceCallStatus.FAILED;
      this.updateState();
    }
  }

  private async handleIceCandidate(data: any) {
    try {
      if (this.state.peerConnection && data.candidate) {
        await this.state.peerConnection.addIceCandidate(new RTCIceCandidate(data.candidate));
      }
    } catch (error) {
      console.error('Failed to handle ICE candidate:', error);
    }
  }

  private handleVoiceCallEnd(data: any) {
    console.log('收到语音通话结束消息:', data);
    this.endCall();
  }

  private startQualityMonitoring() {
    this.qualityCheckInterval = window.setInterval(async () => {
      if (this.state.peerConnection && this.state.status === VoiceCallStatus.CONNECTED) {
        const quality = await this.assessNetworkQuality();
        this.state.networkQuality = quality;
        this.updateState();

        // 检查是否需要切换连接模式
        if (this.state.mode === ConnectionMode.P2P) {
          if (quality.latency > this.config.maxLatency || quality.packetLoss > this.config.maxPacketLoss) {
            console.log('Switching to server relay mode due to poor network quality');
            this.switchToServerRelay();
          }
        }
      }
    }, 2000); // 每2秒检查一次，更快响应网络变化
  }

  private stopQualityMonitoring() {
    if (this.qualityCheckInterval) {
      clearInterval(this.qualityCheckInterval);
      this.qualityCheckInterval = null;
    }
  }

  private async assessNetworkQuality(): Promise<NetworkQuality> {
    // 模拟网络质量评估
    // 实际应用中，应该使用RTCPeerConnection的getStats()方法获取真实的网络质量数据
    const latency = Math.floor(Math.random() * 100) + 30; // 30-130ms (减少模拟延迟)
    const packetLoss = Math.random() * 5; // 0-5% (减少模拟丢包)
    const jitter = Math.floor(Math.random() * 30) + 5; // 5-35ms (减少模拟抖动)

    const recommendedMode = (latency <= 100 && packetLoss <= 1) ? ConnectionMode.P2P : ConnectionMode.SERVER_RELAY;

    return {
      latency,
      packetLoss,
      jitter,
      recommendedMode
    };
  }

  private async switchToServerRelay() {
    // 实现服务器中转模式的切换逻辑
    console.log('Switching to server relay mode');
    // 清除可能存在的失败定时器，避免状态冲突
    if (this.connectionStateTimer) {
      clearTimeout(this.connectionStateTimer);
      this.connectionStateTimer = null;
    }
    // 先更新状态为连接中，避免显示冲突
    this.state.status = VoiceCallStatus.CONNECTING;
    this.updateState();
    
    // 模拟服务器中转连接建立过程
    setTimeout(() => {
      this.state.mode = ConnectionMode.SERVER_RELAY;
      this.state.status = VoiceCallStatus.CONNECTED;
      this.updateState();
      console.log('Server relay mode established');
    }, 1000);
    
    // 这里应该实现与服务器建立中转连接的逻辑
    // 由于是模拟，我们只更新状态
  }

  private updateState() {
    this.stateChangeCallbacks.forEach(callback => {
      callback({ ...this.state });
    });
  }

  // 发起语音通话
  async initiateCall(userId: string, remoteUserId: string) {
    try {
      this.state.userId = userId;
      this.state.remoteUserId = remoteUserId;
      this.state.status = VoiceCallStatus.CONNECTING;
      this.updateState();

      // 获取本地流
      await this.getLocalStream();

      // 创建对等连接
      this.state.peerConnection = this.createPeerConnection();
      this.state.mode = ConnectionMode.P2P;

      // 创建offer
      const offer = await this.state.peerConnection.createOffer({
        offerToReceiveAudio: true,
        offerToReceiveVideo: false
      });
      await this.state.peerConnection.setLocalDescription(offer);

      // 生成通话ID
      this.state.callId = `call_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      // 发送offer
      websocketService.send({
        type: 'voice_call_offer',
        call_id: this.state.callId,
        offer: offer,
        sender_id: userId,
        receiver_id: remoteUserId
      });

      // 开始网络质量监测
      this.startQualityMonitoring();

      return this.state.callId;
    } catch (error) {
      console.error('Failed to initiate voice call:', error);
      this.state.status = VoiceCallStatus.FAILED;
      this.updateState();
      throw error;
    }
  }

  // 结束语音通话
  endCall() {
    if (this.state.callId) {
      websocketService.send({
        type: 'voice_call_end',
        call_id: this.state.callId,
        remote_user_id: this.state.remoteUserId
      });
    }

    // 关闭对等连接
    if (this.state.peerConnection) {
      this.state.peerConnection.close();
      this.state.peerConnection = null;
    }

    // 停止本地流
    if (this.state.localStream) {
      this.state.localStream.getTracks().forEach(track => track.stop());
      this.state.localStream = null;
    }

    // 停止远程流
    if (this.state.remoteStream) {
      this.state.remoteStream.getTracks().forEach(track => track.stop());
      this.state.remoteStream = null;
    }

    // 停止质量监测
    this.stopQualityMonitoring();
    
    // 清除连接状态定时器
    if (this.connectionStateTimer) {
      clearTimeout(this.connectionStateTimer);
      this.connectionStateTimer = null;
    }

    // 更新状态
    this.state.status = VoiceCallStatus.IDLE;
    this.state.mode = null;
    this.state.networkQuality = null;
    this.state.callId = null;
    this.state.remoteUserId = null;
    this.state.endTime = Date.now();
    this.updateState();
  }

  // 手动切换连接模式
  async switchConnectionMode(mode: ConnectionMode) {
    if (this.state.status !== VoiceCallStatus.CONNECTED) {
      throw new Error('Call is not connected');
    }

    this.state.mode = mode;
    this.updateState();

    if (mode === ConnectionMode.P2P) {
      // 重新建立P2P连接
      await this.reestablishP2PConnection();
    } else if (mode === ConnectionMode.SERVER_RELAY) {
      // 切换到服务器中转模式
      await this.switchToServerRelay();
    }
  }

  private async reestablishP2PConnection() {
    // 实现重新建立P2P连接的逻辑
    console.log('Reestablishing P2P connection');
    // 由于是模拟，我们只更新状态
  }

  // 评估网络环境
  async assessNetworkEnvironment(): Promise<NetworkQuality> {
    const quality = await this.assessNetworkQuality();
    this.state.networkQuality = quality;
    this.updateState();
    return quality;
  }

  // 获取当前状态
  getState(): VoiceCallState {
    return { ...this.state };
  }

  // 设置状态变化回调
  onStateChange(callback: (state: VoiceCallState) => void) {
    this.stateChangeCallbacks.push(callback);
  }

  // 设置配置
  setConfig(config: Partial<VoiceCallConfig>) {
    this.config = { ...this.config, ...config };
  }

  // 重新注册WebSocket监听器
  registerWebSocketListeners() {
    this.setupWebSocketListeners();
  }
}

export const voiceCallService = new VoiceCallService();
