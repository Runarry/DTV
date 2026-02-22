import { invoke } from '@tauri-apps/api/core';

import { Platform } from './types';

export interface StartFlvProxySessionArgs {
  upstreamUrl: string;
  platform: Platform;
  roomId?: string | null;
}

interface StartFlvProxySessionPayload {
  upstreamUrl: string;
  platform: string;
  roomId?: string | null;
}

interface StartFlvProxySessionResponse {
  sessionId: string;
  proxyUrl: string;
}

export async function startFlvProxySession(args: StartFlvProxySessionArgs): Promise<StartFlvProxySessionResponse> {
  const payload: StartFlvProxySessionPayload = {
    upstreamUrl: args.upstreamUrl,
    platform: String(args.platform),
    roomId: args.roomId ?? null,
  };
  const result = await invoke<StartFlvProxySessionResponse>('start_flv_proxy_session', { payload });
  if (!result?.sessionId || !result?.proxyUrl) {
    throw new Error('启动 FLV 代理会话失败：返回结果不完整');
  }
  return result;
}

export async function stopFlvProxySession(sessionId: string): Promise<void> {
  if (!sessionId) {
    return;
  }
  await invoke('stop_flv_proxy_session', { sessionId });
}
