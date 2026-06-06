<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

const genericToast = ref<string | null>(null);
const toastId = ref(0);

let unlistenToast: UnlistenFn | null = null;
let toastTimeout: any = null;

onMounted(async () => {
  unlistenToast = await listen<{message: string}>('show_toast', (event) => {
    genericToast.value = event.payload.message;
    toastId.value++;
    if (toastTimeout) {
      clearTimeout(toastTimeout);
    }
    toastTimeout = setTimeout(() => {
      genericToast.value = null;
    }, 3000);
  });
});

onUnmounted(() => {
  if (unlistenToast) unlistenToast();
  if (toastTimeout) clearTimeout(toastTimeout);
});
</script>

<template>
  <div class="notifications-wrapper">
    <Transition name="toast">
      <div v-if="genericToast" :key="'toast-' + toastId" class="notification-toast simple-toast">
        <div class="toast-header">
          <span class="task-name">{{ genericToast }}</span>
        </div>
        <div class="progress-bar-bg timeout-bg">
          <div class="progress-bar-fill timeout-fill"></div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.notifications-wrapper {
  position: fixed;
  top: 40px;
  right: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  z-index: 1000;
}

.notification-toast {
  position: relative;
  overflow: hidden;
  width: 320px;
  background-color: var(--bg-shell);
  border: 1px solid var(--border-line);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 10px 30px color-mix(in srgb, var(--color-black) 50%, transparent);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.toast-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-main);
}

.progress-bar-bg {
  width: 100%;
  height: 6px;
  background-color: var(--surface-1);
  border-radius: 3px;
  overflow: hidden;
}

.timeout-bg {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 4px;
  border-radius: 0;
  margin: 0;
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--accent);
  transition: width 0.15s linear;
}

.timeout-fill {
  width: 100%;
  transition: none;
  animation: shrink 3s linear forwards;
}

@keyframes shrink {
  from { width: 100%; }
  to { width: 0%; }
}

.toast-enter-active,
.toast-leave-active {
  transition: opacity 0.3s, transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
