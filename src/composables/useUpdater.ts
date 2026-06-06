import { check } from '@tauri-apps/plugin-updater';
import { exit } from '@tauri-apps/plugin-process';
import { emit } from '@tauri-apps/api/event';

import { ref } from 'vue';

let isChecking = false;

export const updateState = ref<'idle' | 'checking' | 'downloading' | 'restarting'>('idle');
export const updateProgress = ref(0);

export async function checkForUpdates(manual = false) {
  if (isChecking) return;
  
  if (!navigator.onLine) {
    if (manual) {
      emit('show_toast', { message: 'No internet connection' });
    }
    return;
  }

  isChecking = true;
  updateState.value = 'checking';
  updateProgress.value = 0;

  try {
    if (manual) {
      // Add a small delay so the splash screen doesn't just flash if the check is very fast
      await new Promise(resolve => setTimeout(resolve, 600));
    }
    const update = await check();

    if (update) {
      updateState.value = 'downloading';
      updateProgress.value = 0;

      let downloaded = 0;
      let contentLength = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength || 0;
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              const progress = Math.round((downloaded / contentLength) * 100);
              updateProgress.value = progress;
            }
            break;
          case 'Finished':
            break;
        }
      });

      updateState.value = 'restarting';
      updateProgress.value = 100;

      setTimeout(async () => {
        await exit(0);
      }, 2000);
    } else {
      updateState.value = 'idle';
      if (manual) {
        emit('show_toast', { message: 'You have the latest version!' });
      }
    }
  } catch (error: any) {
    console.error('Update failed:', error);
    updateState.value = 'idle';
    if (manual) {
      const msg = error?.message || String(error) || 'Unknown error';
      emit('show_toast', { message: `Update check failed: ${msg}` });
    }
  } finally {
    isChecking = false;
  }
}
