import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { DesktopClient, DesktopSnapshot } from '@instancedeck/ui';
import { createPreviewClient } from '@instancedeck/ui';

function request<T>(value: Record<string, unknown>): Promise<T> {
  return invoke<T>('desktop_request', { request: value });
}
const nativeClient: DesktopClient = {
  snapshot: (refresh = false) => request({ type: 'snapshot', refresh }),
  subscribe: async (listener, onError) => {
    const stopStatus = await listen<DesktopSnapshot>(
      'desktop-status',
      (event) => listener(event.payload),
    );
    try {
      const stopError = await listen<string>('desktop-error', (event) =>
        onError(event.payload),
      );
      return () => {
        stopStatus();
        stopError();
      };
    } catch (error) {
      stopStatus();
      throw error;
    }
  },
  create: (input) => request({ type: 'create', input }),
  update: (profile) => request({ type: 'update', profile }),
  action: (id, action, confirmation) =>
    request({ type: 'action', id, action, confirmation }),
  mainAction: (appId, action) => request({ type: 'mainAction', appId, action }),
  saveSettings: (settings) => request({ type: 'saveSettings', settings }),
  diagnostics: (redact) => request({ type: 'diagnostics', redact }),
  plan: (id) => request({ type: 'plan', id }),
  browse: () => request({ type: 'browse' }),
  window: (action) => request({ type: 'window', action }),
};
export const desktopClient = isTauri() ? nativeClient : createPreviewClient();
