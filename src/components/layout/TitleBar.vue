<script setup lang="ts">
import { ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { checkForUpdates } from '../../composables/useUpdater';
import { t } from '../../composables/useI18n';

const appWindow = getCurrentWindow();

const hoveredTooltip = ref('');
const tooltipLeft = ref(0);

const handleMouseOver = (e: MouseEvent) => {
  const target = (e.target as HTMLElement).closest('[data-tooltip]');
  if (target) {
    hoveredTooltip.value = target.getAttribute('data-tooltip') || '';
    const rect = target.getBoundingClientRect();
    tooltipLeft.value = rect.left + rect.width / 2;
  }
};

const handleMouseOut = (e: MouseEvent) => {
  const target = (e.target as HTMLElement).closest('[data-tooltip]');
  const related = e.relatedTarget as Node | null;
  if (target && related && target.contains(related)) {
    return;
  }
  hoveredTooltip.value = '';
};

const handleUpdateCheck = () => {
  checkForUpdates(true);
};

const minimize = () => {
  appWindow.minimize();
};

const toggleMaximize = () => {
  appWindow.toggleMaximize();
};

const close = () => {
  appWindow.close();
};

const startDrag = (e: MouseEvent) => {

  const target = e.target as HTMLElement;
  if (!target.closest('.window-control')) {
    appWindow.startDragging();
  }
};
</script>

<template>
  <div class="titlebar" @mousedown="startDrag" @mouseover="handleMouseOver" @mouseout="handleMouseOut">
    <div class="titlebar-left">
      <img src="/icon.svg" onerror="this.src='/vite.svg'" alt="" class="app-icon" />
      <span class="app-title">visual <span class="text-accent">client</span></span>
      <div class="title-separator"></div>
      <span class="app-author">by kaqvu</span>
    </div>
    
    <div class="titlebar-controls">
      <div class="window-control update-control" @click="handleUpdateCheck" :data-tooltip="t('updater.check_tooltip')">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" style="pointer-events: none;">
          <path d="M4 12V10a4 4 0 0 1 4-4h12"></path>
          <polyline points="16 2 20 6 16 10"></polyline>
          <path d="M20 12V14a4 4 0 0 1-4 4H4"></path>
          <polyline points="8 14 4 18 8 22"></polyline>
        </svg>
      </div>
      <div class="controls-separator"></div>
      <div class="window-control" @click="minimize">
        <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/01/svg">
          <rect x="2" y="5" width="8" height="2" rx="1" fill="currentColor"/>
        </svg>
      </div>
      <div class="window-control" @click="toggleMaximize">
        <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/01/svg">
          <rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.5"/>
        </svg>
      </div>
      <div class="window-control close-control" @click="close">
        <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/01/svg">
          <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
    </div>

    <Transition name="tooltip-fade">
      <div v-if="hoveredTooltip" class="titlebar-tooltip" :style="{ left: tooltipLeft + 'px' }">
        {{ hoveredTooltip }}
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.titlebar {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px 0 16px;
  user-select: none;
  background-color: transparent;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 4px;
}

.app-icon {
  width: 26px;
  height: 26px;
  pointer-events: none;
}

.app-title {
  font-weight: 600;
  font-size: 16px;
  letter-spacing: 0.5px;
  pointer-events: none;
}

.text-accent {
  color: var(--accent);
}

.title-separator {
  width: 1px;
  height: 16px;
  background-color: var(--border-line);
  margin: 0 8px;
}

.app-author {
  font-size: 13px;
  color: var(--text-muted);
  font-weight: 500;
  pointer-events: none;
}

.titlebar-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.controls-separator {
  width: 1px;
  height: 16px;
  background-color: var(--border-line);
  margin: 0 4px;
}

.window-control {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  transition: background-color 0.2s ease, color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  cursor: pointer;
}

.window-control:hover {
  background-color: color-mix(in srgb, var(--color-white) 10%, transparent);
  color: var(--text-main);
}

.window-control:active {
  transform: scale(0.85);
}

.close-control:hover {
  background-color: var(--danger);
  color: white;
}

.titlebar-tooltip {
  position: fixed;
  top: 56px;
  background-color: var(--color-black);
  color: var(--color-white);
  font-weight: 700;
  font-size: 13px;
  padding: 6px 12px;
  border-radius: 8px;
  pointer-events: none;
  z-index: 1000;
  box-shadow: 0 4px 6px color-mix(in srgb, var(--color-black) 30%, transparent);
  white-space: nowrap;
  transform: translateX(-50%);
}

.tooltip-fade-enter-active, .tooltip-fade-leave-active {
  transition: opacity 0.2s, transform 0.2s;
}
.tooltip-fade-enter-from, .tooltip-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, -4px);
}
.tooltip-fade-enter-to, .tooltip-fade-leave-from {
  opacity: 1;
  transform: translate(-50%, 0);
}
</style>
