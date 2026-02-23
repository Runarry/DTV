import { invoke } from '@tauri-apps/api/core';
import { Platform } from './types';

export type RecordingStatus = 'starting' | 'recording' | 'reconnecting' | 'stopped' | 'failed';

export interface StartLiveRecordingPayload {
  platform: Platform;
  roomId: string;
  quality?: string;
  segmentMinutes?: number;
  outputDir?: string | null;
  cookie?: string | null;
}

export interface StartLiveRecordingResponse {
  taskId: string;
  resolvedStreamUrl: string;
  outputDir: string;
  startedAt: number;
}

export interface RecordingTaskSnapshot {
  taskId: string;
  platform: string;
  roomId: string;
  quality: string;
  status: RecordingStatus | string;
  outputDir: string;
  currentFile?: string | null;
  segmentIndex: number;
  bytesWritten: number;
  startedAt: number;
  updatedAt: number;
  message?: string | null;
}

export interface RecordingStatusEventPayload {
  taskId: string;
  platform: string;
  roomId: string;
  status: RecordingStatus | string;
  currentFile?: string | null;
  segmentIndex: number;
  bytesWritten: number;
  message?: string | null;
  timestamp: number;
}

export async function getRecordingDefaultOutputDir(): Promise<string> {
  return invoke<string>('get_recording_output_dir_default');
}

export async function listLiveRecordings(): Promise<RecordingTaskSnapshot[]> {
  return invoke<RecordingTaskSnapshot[]>('list_live_recordings');
}

export async function startLiveRecording(
  payload: StartLiveRecordingPayload,
): Promise<StartLiveRecordingResponse> {
  const normalizedPayload = {
    platform: String(payload.platform),
    roomId: payload.roomId,
    quality: payload.quality ?? '原画',
    segmentMinutes: payload.segmentMinutes ?? 30,
    outputDir: payload.outputDir ?? null,
    cookie: payload.cookie ?? null,
  };
  return invoke<StartLiveRecordingResponse>('start_live_recording', { payload: normalizedPayload });
}

export async function stopLiveRecording(taskId: string): Promise<void> {
  await invoke('stop_live_recording', { taskId });
}

export async function stopAllLiveRecordings(): Promise<void> {
  await invoke('stop_all_live_recordings');
}
