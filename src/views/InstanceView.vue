<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../composables/useI18n';
import { useInstances, Instance } from '../composables/useInstances';
import { useAccounts } from '../composables/useAccounts';
import IconInstance from '../components/icons/IconInstance.vue';
import IconPlay from '../components/icons/IconPlay.vue';
import IconGamepad from '../components/icons/IconGamepad.vue';
import IconFolder from '../components/icons/IconFolder.vue';
import LoginRequiredModal from '../components/modals/LoginRequiredModal.vue';

const props = defineProps<{
  instanceId: string
}>();

const emit = defineEmits(['openSettings', 'openAccounts']);

const { instances } = useInstances();
const { accounts } = useAccounts();

const instance = computed(() => {
  return instances.value.find((i: Instance) => i.id === props.instanceId);
});

const activeTab = ref(0);
const servers = ref<{name: string, ip: string, motdHtml?: string, loadingMotd?: boolean, online?: boolean}[]>([]);
const worlds = ref<{folder_name: string, name: string, last_played: number}[]>([]);

const loadData = async () => {
  if (instance.value) {
    try {
      const fetchedServers: any[] = await invoke('get_instance_servers', { id: instance.value.id });
      servers.value = fetchedServers.map(s => ({
        ...s,
        motdHtml: '',
        loadingMotd: true,
        online: false
      }));
      
      servers.value.forEach(async (server, index) => {
        try {
          const res = await fetch(`https://api.mcsrvstat.us/3/${encodeURIComponent(server.ip)}`);
          const data = await res.json();
          if (data.online) {
            servers.value[index].motdHtml = data.motd.html.join('<br>');
            servers.value[index].online = true;
          } else {
            servers.value[index].motdHtml = '<span style="color: var(--error);">Server is offline</span>';
          }
        } catch(e) {
          servers.value[index].motdHtml = '<span style="color: var(--error);">Failed to ping server</span>';
        }
        servers.value[index].loadingMotd = false;
      });

      worlds.value = await invoke('get_instance_worlds', { id: instance.value.id });
    } catch (e) {
      console.error(e);
    }
  }
};

onMounted(loadData);
watch(() => props.instanceId, loadData);

const showLoginModal = ref(false);

const handlePlay = async () => {
  const currentAccount = accounts.value.find(a => a.active)?.username;
  if (!currentAccount) {
    showLoginModal.value = true;
  } else {
    try {
      await invoke('launch_instance', { id: instance.value!.id, username: currentAccount, launchingText: t('instance.launching') });
    } catch (e) {
      console.error(e);
    }
  }
};

const handleAddAccountClick = () => {
  showLoginModal.value = false;
  emit('openAccounts');
};

const openFolder = async () => {
  if (instance.value) {
    try {
      await invoke('open_instance_folder', { id: instance.value.id });
    } catch (e) {
      console.error(e);
    }
  }
};
</script>

<template>
  <div v-if="instance" class="instance-view">
    <header class="instance-header">
      <div class="header-left">
        <div class="library-icon">
          <IconInstance class="library-icon-svg" />
        </div>
        <div class="instance-info">
          <h1 class="title">{{ instance.name }}</h1>
          <div class="instance-loader">
            <IconGamepad class="gamepad-icon" />
            <span class="subtitle">{{ instance.loader.charAt(0).toUpperCase() + instance.loader.slice(1) }} {{ instance.version }}</span>
          </div>
        </div>
      </div>
      <div class="actions">
        <button class="btn-play" @click="handlePlay">
          <IconPlay class="play-icon" />
          {{ t('instance.play') || 'Play' }}
        </button>
        <button class="btn-settings" @click="emit('openSettings', instance)">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
          </svg>
        </button>
        <button class="btn-settings" @click="openFolder">
          <IconFolder class="folder-icon" />
        </button>
      </div>
    </header>

    <div class="tabs-container">
      <div class="tab-indicator" :style="{ transform: `translateX(${activeTab * 100}%)` }"></div>
      
      <div 
        class="tab-button" 
        :class="{ active: activeTab === 0 }"
        @click="activeTab = 0"
      >
        <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
          <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
          <line x1="6" y1="6" x2="6.01" y2="6"></line>
          <line x1="6" y1="18" x2="6.01" y2="18"></line>
        </svg>
        <span>{{ t('instance.servers') }}</span>
      </div>

      <div 
        class="tab-button" 
        :class="{ active: activeTab === 1 }"
        @click="activeTab = 1"
      >
        <svg class="tab-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M6.15407 7.30116C7.52877 5.59304 9.63674 4.5 12 4.5C12.365 4.5 12.7238 4.52607 13.0748 4.57644L13.7126 5.85192L11.2716 8.2929L8.6466 8.6679L7.36009 9.95441L6.15407 7.30116ZM5.2011 8.82954C4.75126 9.79256 4.5 10.8669 4.5 12C4.5 15.6945 7.17133 18.7651 10.6878 19.3856L11.0989 18.7195L8.8147 15.547L10.3741 13.5256L9.63268 13.1549L6.94027 13.6036L6.41366 11.4972L5.2011 8.82954ZM7.95559 11.4802L8.05962 11.8964L9.86722 11.5951L11.3726 12.3478L14.0824 11.9714L18.9544 14.8135C19.3063 13.9447 19.5 12.995 19.5 12C19.5 8.93729 17.6642 6.30336 15.033 5.13856L15.5377 6.1481L11.9787 9.70711L9.35371 10.0821L7.95559 11.4802ZM18.2539 16.1414C16.9774 18.0652 14.8369 19.366 12.3859 19.4902L12.9011 18.6555L10.6853 15.578L12.0853 13.7632L13.7748 13.5286L18.2539 16.1414ZM12 3C7.02944 3 3 7.02944 3 12C3 16.9706 7.02944 21 12 21C16.9706 21 21 16.9706 21 12C21 7.02944 16.9706 3 12 3Z" fill="currentColor"/>
        </svg>
        <span>{{ t('instance.worlds') }}</span>
      </div>
    </div>

    <div class="tab-content">
      <div v-if="activeTab === 0 && servers.length === 0" class="empty-message">{{ t('instance.no_servers') }}</div>
      <div v-else-if="activeTab === 0" class="servers-list">
        <div class="server-card" v-for="server in servers" :key="server.ip">
          <div class="item-icon server-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
              <line x1="6" y1="6" x2="6.01" y2="6"></line>
              <line x1="6" y1="18" x2="6.01" y2="18"></line>
            </svg>
          </div>
          <div class="server-card-content">
            <div class="server-info">
              <h4 class="item-name">{{ server.name }}</h4>
              <div class="server-motd" v-if="server.loadingMotd">Pinging...</div>
              <div class="server-motd" v-else v-html="server.motdHtml"></div>
            </div>
            <div class="server-ip">{{ server.ip }}</div>
          </div>
        </div>
      </div>
      
      <div v-if="activeTab === 1 && worlds.length === 0" class="empty-message">{{ t('instance.no_worlds') }}</div>
      <div v-else-if="activeTab === 1" class="worlds-list">
        <div class="item-card" v-for="world in worlds" :key="world.folder_name">
          <div class="item-icon world-icon">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path fill-rule="evenodd" clip-rule="evenodd" d="M6.15407 7.30116C7.52877 5.59304 9.63674 4.5 12 4.5C12.365 4.5 12.7238 4.52607 13.0748 4.57644L13.7126 5.85192L11.2716 8.2929L8.6466 8.6679L7.36009 9.95441L6.15407 7.30116ZM5.2011 8.82954C4.75126 9.79256 4.5 10.8669 4.5 12C4.5 15.6945 7.17133 18.7651 10.6878 19.3856L11.0989 18.7195L8.8147 15.547L10.3741 13.5256L9.63268 13.1549L6.94027 13.6036L6.41366 11.4972L5.2011 8.82954ZM7.95559 11.4802L8.05962 11.8964L9.86722 11.5951L11.3726 12.3478L14.0824 11.9714L18.9544 14.8135C19.3063 13.9447 19.5 12.995 19.5 12C19.5 8.93729 17.6642 6.30336 15.033 5.13856L15.5377 6.1481L11.9787 9.70711L9.35371 10.0821L7.95559 11.4802ZM18.2539 16.1414C16.9774 18.0652 14.8369 19.366 12.3859 19.4902L12.9011 18.6555L10.6853 15.578L12.0853 13.7632L13.7748 13.5286L18.2539 16.1414ZM12 3C7.02944 3 3 7.02944 3 12C3 16.9706 7.02944 21 12 21C16.9706 21 21 16.9706 21 12C21 7.02944 16.9706 3 12 3Z" fill="currentColor"/>
            </svg>
          </div>
          <div class="item-info">
            <h4 class="item-name">{{ world.name || world.folder_name }}</h4>
            <span class="item-sub">{{ world.last_played > 0 ? new Date(world.last_played).toLocaleString() : world.folder_name }}</span>
          </div>
        </div>
      </div>
    </div>

    <Transition name="modal">
      <LoginRequiredModal 
        v-if="showLoginModal" 
        @close="showLoginModal = false" 
        @login="handleAddAccountClick" 
      />
    </Transition>
  </div>
</template>

<style scoped>
.instance-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 0;
}

.instance-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0;
  padding-bottom: 32px;
  border-bottom: 1px solid var(--border-line);
}

.tabs-container {
  display: flex;
  position: relative;
  background-color: var(--surface-hover);
  border-radius: 999px;
  padding: 4px;
  width: fit-content;
  margin-top: 10px;
}

.tab-indicator {
  position: absolute;
  top: 4px;
  bottom: 4px;
  left: 4px;
  width: calc((100% - 8px) / 2);
  background-color: color-mix(in srgb, var(--accent) 15%, transparent);
  border-radius: 999px;
  transition: transform 0.35s cubic-bezier(0.25, 1, 0.5, 1);
}

.tab-button {
  position: relative;
  z-index: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 20px;
  cursor: pointer;
  border-radius: 999px;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.tab-button:active {
  transform: scale(0.92);
}

.tab-button span {
  font-weight: 700;
  color: var(--color-white);
  font-size: 0.9rem;
  transition: color 0.2s ease;
}

.tab-icon {
  width: 14px;
  height: 14px;
  color: var(--text-muted);
  transition: color 0.2s ease;
}

.tab-button.active span,
.tab-button.active .tab-icon {
  color: var(--accent);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 20px;
}

.library-icon {
  width: 72px;
  height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border: 4px solid var(--surface-2);
  border-radius: 16px;
  background-color: var(--surface-1);
}

.library-icon-svg {
  width: 72px;
  height: 72px;
}

.instance-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.title {
  margin: 0;
  font-size: 1.8rem;
  font-weight: 700;
  color: var(--text-main);
}

.instance-loader {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
}

.gamepad-icon {
  width: 18px;
  height: 18px;
  color: var(--text-muted);
}

.subtitle {
  font-size: 1rem;
  color: var(--text-muted);
  font-weight: 600;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-play {
  padding: 0 24px;
  height: 44px;
  background-color: var(--accent);
  color: var(--color-black);
  border: none;
  border-radius: 16px;
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: background-color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.play-icon {
  width: 20px;
  height: 20px;
}

.btn-play:hover {
  background-color: var(--accent-hover);
}

.btn-play:active {
  transform: scale(0.96);
}

.btn-settings {
  width: 48px;
  height: 48px;
  background-color: var(--surface-2);
  color: var(--text-secondary);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s ease, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.folder-icon {
  width: 20px;
  height: 20px;
}

.btn-settings:hover {
  background-color: var(--surface-3);
}

.btn-settings:active {
  transform: scale(0.85);
}

.tab-content {
  margin-top: 24px;
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.worlds-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
  align-content: start;
  flex: 1;
}

.servers-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
}

.item-card, .server-card {
  background-color: var(--bg-shell);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: background-color 0.2s ease, transform 0.2s ease;
  cursor: default;
}

.item-card:hover, .server-card:hover {
  background-color: var(--surface-dark);
}

.server-card-content {
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  overflow: hidden;
}

.server-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  overflow: hidden;
}

.server-motd {
  font-size: 0.85rem;
  color: var(--text-muted);
  white-space: pre-wrap;
  line-height: 1.4;
}

.server-ip {
  font-size: 0.9rem;
  color: var(--text-main);
  background-color: var(--surface-2);
  padding: 4px 10px;
  border-radius: 8px;
  font-family: monospace;
  white-space: nowrap;
  flex-shrink: 0;
}

.item-icon {
  width: 48px;
  height: 48px;
  background-color: var(--surface-2);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  flex-shrink: 0;
}

.server-icon {
  color: #55ff55;
  background-color: color-mix(in srgb, #55ff55 10%, transparent);
}

.world-icon {
  color: #55ffff;
  background-color: color-mix(in srgb, #55ffff 10%, transparent);
}

.item-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  overflow: hidden;
}

.item-name {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 700;
  color: var(--text-main);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-sub {
  font-size: 0.85rem;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-message {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  color: var(--text-muted);
  font-style: italic;
  font-size: 1.1rem;
}
</style>
