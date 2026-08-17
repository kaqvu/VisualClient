<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { checkForUpdates } from '../../composables/useUpdater';
import { t } from '../../composables/useI18n';
import { useInstances } from '../../composables/useInstances';
import IconStop from '../icons/IconStop.vue';

const { instances, runningInstances, killInstance } = useInstances();

const emit = defineEmits(['openInstance']);

const selectedInstanceId = ref<string | null>(null);

const activeInstanceId = computed(() => {
  if (runningInstances.value.length === 0) return null;
  if (selectedInstanceId.value && runningInstances.value.includes(selectedInstanceId.value)) {
    return selectedInstanceId.value;
  }
  return runningInstances.value[0];
});

const activeInstanceName = computed(() => {
  if (!activeInstanceId.value) return '';
  const inst = instances.value.find(i => i.id === activeInstanceId.value);
  return inst ? inst.name : 'Minecraft';
});

const isDropdownOpen = ref(false);

const toggleDropdown = () => {
  if (runningInstances.value.length > 1) {
    isDropdownOpen.value = !isDropdownOpen.value;
  }
};

watch(() => runningInstances.value.length, (newLen) => {
  if (newLen < 2) {
    isDropdownOpen.value = false;
  }
});

const closeDropdown = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.active-instance-tile')) {
    isDropdownOpen.value = false;
  }
};

let unlistenResized: (() => void) | null = null;

onMounted(async () => {
  document.addEventListener('click', closeDropdown);
  await updateMaximized();
  unlistenResized = await appWindow.onResized(() => {
    updateMaximized();
  });
});

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown);
  if (unlistenResized) unlistenResized();
});

const selectInstance = (id: string) => {
  selectedInstanceId.value = id;
  isDropdownOpen.value = false;
};

const handleStopClick = async (e: Event, id?: string) => {
  e.stopPropagation();
  const targetId = id || activeInstanceId.value;
  if (targetId) {
    await killInstance(targetId);
  }
};

const goToInstance = (e: Event, id: string | null) => {
  e.stopPropagation();
  if (id) {
    emit('openInstance', id);
  }
};

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

const isMaximized = ref(false);

const updateMaximized = async () => {
  isMaximized.value = await appWindow.isMaximized();
};

const minimize = () => {
  appWindow.minimize();
};

const toggleMaximize = async () => {
  await appWindow.toggleMaximize();
  updateMaximized();
};

const close = () => {
  appWindow.close();
};

const startDrag = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.window-control') && !target.closest('.active-instance-tile')) {
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
      <span class="app-author">by kacpixcrf</span>
    </div>
    
    <div class="titlebar-controls">
      <div v-if="runningInstances.length > 0" class="active-instance-tile" @click="toggleDropdown">
        <div class="active-circle"></div>
        <span class="active-name" :data-tooltip="t('instance.view_instance') || 'View instance'" @click="goToInstance($event, activeInstanceId)">{{ activeInstanceName }}</span>
        <IconStop class="stop-btn" @click="handleStopClick($event)" />
        <svg v-if="runningInstances.length > 1" class="dropdown-arrow" :class="{ 'is-open': isDropdownOpen }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>

        <Transition name="dropdown">
          <div v-if="isDropdownOpen && runningInstances.length > 1" class="instance-dropdown" @click.stop>
            <div 
              v-for="id in runningInstances" 
              :key="id" 
              class="dropdown-item"
              @click="selectInstance(id)"
            >
              <div class="item-name-wrapper">
                <span class="item-name">{{ instances.find(i => i.id === id)?.name || 'Minecraft' }}</span>
                <svg v-if="id === activeInstanceId" class="active-star" viewBox="0 0 24 24" fill="#ffd700" stroke="#ffd700" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
                </svg>
              </div>
              <IconStop class="stop-btn-small" @click="handleStopClick($event, id)" />
            </div>
          </div>
        </Transition>
      </div>
      <div v-else class="active-instance-tile empty-state">
        <div class="active-circle inactive"></div>
        <span class="active-name inactive-text">{{ t('instance.no_instances') || 'No instances running' }}</span>
      </div>

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
        <svg v-if="!isMaximized" width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/01/svg">
          <rect x="2.5" y="2.5" width="7" height="7" rx="1" stroke="currentColor" stroke-width="1.5"/>
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/01/svg">
          <path d="M4.5 4.5V2.5C4.5 1.94772 4.94772 1.5 5.5 1.5H9.5C10.0523 1.5 10.5 1.94772 10.5 2.5V6.5C10.5 7.05228 10.0523 7.5 9.5 7.5H7.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
          <rect x="1.5" y="4.5" width="6" height="6" rx="1" stroke="currentColor" stroke-width="1.5"/>
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
  font-weight: 600;
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

.active-instance-tile {
  position: relative;
  display: flex;
  align-items: center;
  background-color: color-mix(in srgb, var(--color-white) 3%, transparent);
  border-radius: 12px;
  padding: 4px 8px 4px 8px;
  gap: 8px;
  margin-right: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.active-instance-tile:hover {
  background-color: color-mix(in srgb, var(--bg-shell) 85%, var(--color-white));
}

.active-circle {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: var(--accent);
  box-shadow: 0 0 8px var(--accent);
}

.active-circle.inactive {
  background-color: var(--text-muted);
  box-shadow: none;
}

.active-name {
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
}

.active-name:hover {
  text-decoration: underline;
  color: var(--text-main);
}

.active-name.inactive-text {
  cursor: default;
}

.active-name.inactive-text:hover {
  text-decoration: none;
  color: var(--text-muted);
}

.empty-state {
  cursor: default;
}

.empty-state:hover {
  background-color: color-mix(in srgb, var(--color-white) 3%, transparent);
}

.stop-btn {
  width: 16px;
  height: 16px;
  color: var(--danger);
  transition: transform 0.2s, filter 0.2s;
  cursor: pointer;
}

.stop-btn:hover {
  filter: brightness(1.2);
}

.stop-btn:active {
  transform: scale(0.85);
}

.dropdown-arrow {
  width: 14px;
  height: 14px;
  color: var(--text-muted);
  transition: transform 0.2s;
}

.dropdown-arrow.is-open {
  transform: rotate(180deg);
}

.instance-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  background-color: var(--bg-shell);
  border: 1px solid var(--border-line);
  border-radius: 12px;
  padding: 4px;
  min-width: 180px;
  box-shadow: 0 8px 24px color-mix(in srgb, var(--color-black) 50%, transparent);
  z-index: 1000;
  display: flex;
  flex-direction: column;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
  gap: 16px;
}

.dropdown-item:hover {
  background-color: color-mix(in srgb, var(--color-white) 6%, transparent);
}

.item-name-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.active-star {
  width: 14px;
  height: 14px;
}

.item-name {
  font-size: 13px;
  color: var(--text-main);
  font-weight: 500;
}

.stop-btn-small {
  width: 24px;
  height: 24px;
  padding: 4px;
  border-radius: 50%;
  color: var(--danger);
  cursor: pointer;
  transition: background-color 0.2s ease, color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.stop-btn-small:hover {
  background-color: color-mix(in srgb, var(--danger) 15%, transparent);
}

.stop-btn-small:active {
  transform: scale(0.85);
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.2s, transform 0.2s;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
