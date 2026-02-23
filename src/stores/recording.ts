import { defineStore } from 'pinia';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { Platform } from '../platforms/common/types';
import type {
  RecordingStatusEventPayload,
  RecordingTaskSnapshot,
  StartLiveRecordingPayload,
} from '../platforms/common/recording';
import {
  getRecordingDefaultOutputDir,
  listLiveRecordings,
  startLiveRecording,
  stopAllLiveRecordings,
  stopLiveRecording,
} from '../platforms/common/recording';

let unlistenRecordingStatus: UnlistenFn | null = null;

const activeStatuses = new Set(['starting', 'recording', 'reconnecting']);

const roomKeyOf = (platform: string | Platform, roomId: string): string => {
  return `${String(platform).toUpperCase()}:${roomId}`;
};

export const useRecordingStore = defineStore('recording', {
  state: () => ({
    tasks: [] as RecordingTaskSnapshot[],
    defaultOutputDir: '' as string,
    initialized: false as boolean,
  }),

  getters: {
    activeTasks: (state) => state.tasks.filter((task) => activeStatuses.has(String(task.status))),
    findTaskByRoom: (state) => (platform: Platform | string, roomId: string) => {
      const key = roomKeyOf(platform, roomId);
      return state.tasks.find((task) => roomKeyOf(task.platform, task.roomId) === key);
    },
    findActiveTaskByRoom: (state) => (platform: Platform | string, roomId: string) => {
      const key = roomKeyOf(platform, roomId);
      return state.tasks.find((task) => roomKeyOf(task.platform, task.roomId) === key && activeStatuses.has(String(task.status)));
    },
  },

  actions: {
    async initialize() {
      if (this.initialized) {
        return;
      }
      this.initialized = true;
      try {
        this.defaultOutputDir = await getRecordingDefaultOutputDir();
      } catch (error) {
        console.warn('[RecordingStore] Failed to load default output dir:', error);
      }

      await this.refreshTasks();

      if (!unlistenRecordingStatus) {
        unlistenRecordingStatus = await listen<RecordingStatusEventPayload>('recording-status', (event) => {
          if (!event.payload) return;
          this.applyStatusEvent(event.payload);
        });
      }
    },

    async refreshTasks() {
      try {
        const list = await listLiveRecordings();
        this.tasks = list.sort((a, b) => b.startedAt - a.startedAt);
      } catch (error) {
        console.error('[RecordingStore] Failed to refresh tasks:', error);
      }
    },

    applyStatusEvent(payload: RecordingStatusEventPayload) {
      const index = this.tasks.findIndex((task) => task.taskId === payload.taskId);
      if (index >= 0) {
        const previous = this.tasks[index];
        this.tasks[index] = {
          ...previous,
          status: payload.status,
          currentFile: payload.currentFile ?? previous.currentFile ?? null,
          segmentIndex: payload.segmentIndex ?? previous.segmentIndex ?? 0,
          bytesWritten: payload.bytesWritten ?? previous.bytesWritten ?? 0,
          message: payload.message ?? previous.message ?? null,
          updatedAt: payload.timestamp ?? Date.now(),
        };
      } else {
        this.tasks.unshift({
          taskId: payload.taskId,
          platform: String(payload.platform).toUpperCase(),
          roomId: payload.roomId,
          quality: '原画',
          status: payload.status,
          outputDir: this.defaultOutputDir || '',
          currentFile: payload.currentFile ?? null,
          segmentIndex: payload.segmentIndex ?? 0,
          bytesWritten: payload.bytesWritten ?? 0,
          startedAt: payload.timestamp ?? Date.now(),
          updatedAt: payload.timestamp ?? Date.now(),
          message: payload.message ?? null,
        });
      }
    },

    async startRecording(options: {
      platform: Platform;
      roomId: string;
      quality?: string;
      outputDir?: string | null;
      cookie?: string | null;
      segmentMinutes?: number;
    }) {
      const activeTask = this.findActiveTaskByRoom(options.platform, options.roomId);
      if (activeTask) {
        return activeTask;
      }
      const payload: StartLiveRecordingPayload = {
        platform: options.platform,
        roomId: options.roomId,
        quality: options.quality ?? '原画',
        outputDir: options.outputDir ?? null,
        cookie: options.cookie ?? null,
        segmentMinutes: options.segmentMinutes ?? 30,
      };
      const response = await startLiveRecording(payload);
      await this.refreshTasks();
      const createdTask = this.tasks.find((task) => task.taskId === response.taskId);
      if (createdTask) {
        return createdTask;
      }
      return this.findTaskByRoom(options.platform, options.roomId) || null;
    },

    async stopTask(taskId: string) {
      if (!taskId) return;
      await stopLiveRecording(taskId);
      this.tasks = this.tasks.filter((task) => task.taskId !== taskId);
    },

    async stopRecordingByRoom(platform: Platform, roomId: string) {
      const task = this.findActiveTaskByRoom(platform, roomId);
      if (!task) {
        return;
      }
      await this.stopTask(task.taskId);
    },

    async stopAll() {
      await stopAllLiveRecordings();
      this.tasks = [];
    },
  },
});
