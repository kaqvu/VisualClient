<script setup lang="ts">
import { ref, computed, onMounted, watch, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../composables/useI18n';
import { useInstances, Instance } from '../composables/useInstances';
import { useAccounts } from '../composables/useAccounts';
import IconInstance from '../components/icons/IconInstance.vue';
import IconPlay from '../components/icons/IconPlay.vue';
import IconGamepad from '../components/icons/IconGamepad.vue';
import IconFolder from '../components/icons/IconFolder.vue';
import IconTrash from '../components/icons/IconTrash.vue';
import IconPlus from '../components/icons/IconPlus.vue';
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
const servers = ref<{name: string, ip: string, accept_textures?: number, motdHtml?: string, loadingMotd?: boolean, online?: boolean}[]>([]);
const worlds = ref<{folder_name: string, name: string, last_played: number, icon_base64?: string}[]>([]);

const activeMenu = ref<string | null>(null);

const toggleMenu = (ip: string) => {
  if (activeMenu.value === ip) {
    activeMenu.value = null;
  } else {
    activeMenu.value = ip;
  }
};

const closeMenu = () => {
  activeMenu.value = null;
};

onMounted(() => {
  document.addEventListener('click', closeMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeMenu);
});

const copyAddress = async (ip: string) => {
  activeMenu.value = null;
  try {
    await navigator.clipboard.writeText(ip);
  } catch(e) {}
};

const isDeleteModalOpen = ref(false);
const serverToDelete = ref<string | null>(null);

const removeServer = (ip: string) => {
  activeMenu.value = null;
  serverToDelete.value = ip;
  isDeleteModalOpen.value = true;
};

const closeDeleteModal = () => {
  isDeleteModalOpen.value = false;
  serverToDelete.value = null;
};

const confirmRemoveServer = async () => {
  if (!serverToDelete.value || !instance.value) return;
  try {
    await invoke('remove_instance_server', { id: instance.value.id, ipToRemove: serverToDelete.value });
    servers.value = servers.value.filter(s => s.ip !== serverToDelete.value);
    closeDeleteModal();
  } catch(e) {
    console.error(e);
  }
};

const isEditModalOpen = ref(false);
const isRpSelectOpen = ref(false);
const isAddingServer = ref(false);
const editServerForm = ref({
  originalIp: '',
  name: '',
  ip: '',
  acceptTextures: null as number | null
});

const openAddServer = () => {
  activeMenu.value = null;
  isAddingServer.value = true;
  editServerForm.value = {
    originalIp: '',
    name: '',
    ip: '',
    acceptTextures: null
  };
  isEditModalOpen.value = true;
};

const openEditServer = (server: any) => {
  activeMenu.value = null;
  isAddingServer.value = false;
  editServerForm.value = {
    originalIp: server.ip,
    name: server.name,
    ip: server.ip,
    acceptTextures: server.accept_textures !== undefined ? server.accept_textures : null
  };
  isEditModalOpen.value = true;
};

const closeEditModal = () => {
  isEditModalOpen.value = false;
};

const saveServer = async () => {
  if (!instance.value || !editServerForm.value.ip || editServerForm.value.ip.trim().length < 1) return;
  try {
    const finalName = editServerForm.value.name || 'Minecraft Server';
    if (isAddingServer.value) {
      await invoke('add_instance_server', {
        id: instance.value.id,
        name: finalName,
        ip: editServerForm.value.ip,
        acceptTextures: editServerForm.value.acceptTextures
      });
      servers.value.push({
        name: finalName,
        ip: editServerForm.value.ip,
        accept_textures: editServerForm.value.acceptTextures,
        loadingMotd: true,
        online: false,
        motdHtml: ''
      });
      isEditModalOpen.value = false;
      fetchMotd(servers.value.length - 1);
    } else {
      await invoke('update_instance_server', {
        id: instance.value.id,
        originalIp: editServerForm.value.originalIp,
        newName: finalName,
        newIp: editServerForm.value.ip,
        acceptTextures: editServerForm.value.acceptTextures
      });
      
      isEditModalOpen.value = false;
      
      const idx = servers.value.findIndex(s => s.ip === editServerForm.value.originalIp);
      if (idx !== -1) {
        servers.value[idx].name = finalName;
        servers.value[idx].ip = editServerForm.value.ip;
        servers.value[idx].accept_textures = editServerForm.value.acceptTextures;
        servers.value[idx].loadingMotd = true;
        fetchMotd(idx);
      }
    }
  } catch(e) {
    console.error(e);
  }
};

const loadData = async () => {
  if (instance.value) {
    try {
      const fetchedServers: any[] = await invoke('get_instance_servers', { id: instance.value.id });
      
      const newServers = fetchedServers.map(s => {
        const existing = servers.value.find(ex => ex.ip === s.ip);
        if (existing) return existing;
        return {
          ...s,
          motdHtml: '',
          loadingMotd: true,
          online: false,
          icon_base64: s.icon_base64 || null
        };
      });

      const ipsToFetch = newServers.filter(s => s.loadingMotd).map(s => s.ip);
      servers.value = newServers;

      servers.value.forEach((server, index) => {
        if (ipsToFetch.includes(server.ip)) {
          fetchMotd(index);
        }
      });

      worlds.value = await invoke('get_instance_worlds', { id: instance.value.id });
    } catch (e) {
      console.error(e);
    }
  }
};

const fetchMotd = async (index: number) => {
  const server = servers.value[index];
  if (!server) return;
  try {
    servers.value[index].loadingMotd = true;
    const res = await fetch(`https://api.mcsrvstat.us/3/${encodeURIComponent(server.ip)}`);
    const data = await res.json();
    
    if (servers.value[index]) {
      if (data.online) {
        servers.value[index].motdHtml = data.motd.html.join('<br>');
        servers.value[index].online = true;
        
        if (data.icon) {
          const cleanBase64 = data.icon.startsWith('data:image/') ? data.icon.split(',')[1] : data.icon;
          if (servers.value[index].icon_base64 !== cleanBase64) {
            servers.value[index].icon_base64 = cleanBase64;
            invoke('update_server_icon', { 
              id: instance.value.id, 
              ipToMatch: server.ip, 
              iconBase64: data.icon 
            }).catch(console.error);
          }
        }
      } else {
        servers.value[index].motdHtml = '';
        servers.value[index].online = false;
      }
    }
  } catch(e) {
    if (servers.value[index]) {
      servers.value[index].motdHtml = '';
      servers.value[index].online = false;
    }
  } finally {
    if (servers.value[index]) {
      servers.value[index].loadingMotd = false;
    }
  }
};

const handleIconError = (e: Event) => {
  const target = e.target as HTMLImageElement;
  target.src = "/server.png";
};

const refreshServers = () => {
  servers.value.forEach((s, idx) => fetchMotd(idx));
};

let pollInterval: number | null = null;

onMounted(() => {
  loadData();
  pollInterval = window.setInterval(() => {
    loadData();
  }, 2000);
});

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval);
});

watch(() => props.instanceId, () => {
  servers.value = [];
  worlds.value = [];
  loadData();
});

const showLoginModal = ref(false);

const handlePlay = async () => {
  const currentAccount = accounts.value.find(a => a.active)?.username;
  if (!currentAccount) {
    showLoginModal.value = true;
  } else {
    try {
      await invoke('launch_instance', { 
        id: instance.value!.id, 
        username: currentAccount, 
        launchingText: t('instance.launching'),
        serverIp: null,
        worldFolder: null
      });
    } catch (e) {
      console.error(e);
    }
  }
};

const isWorldQuickPlaySupported = computed(() => {
  if (!instance.value || !instance.value.version) return false;
  const parts = instance.value.version.split('.');
  if (parts.length >= 2) {
    const major = parseInt(parts[0], 10);
    const minor = parseInt(parts[1], 10);
    return major > 1 || (major === 1 && minor >= 20);
  }
  return false;
});

const handlePlayServer = async (ip: string) => {
  const currentAccount = accounts.value.find(a => a.active)?.username;
  if (!currentAccount) {
    showLoginModal.value = true;
  } else {
    try {
      await invoke('launch_instance', { 
        id: instance.value!.id, 
        username: currentAccount, 
        launchingText: t('instance.launching'),
        serverIp: ip,
        worldFolder: null
      });
    } catch (e) {
      console.error(e);
    }
  }
};

const handlePlayWorld = async (folderName: string) => {
  const currentAccount = accounts.value.find(a => a.active)?.username;
  if (!currentAccount) {
    showLoginModal.value = true;
  } else {
    try {
      await invoke('launch_instance', { 
        id: instance.value!.id, 
        username: currentAccount, 
        launchingText: t('instance.launching'),
        serverIp: null,
        worldFolder: folderName
      });
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

const formatLastPlayed = (timestamp: number) => {
  if (!timestamp) return t('instance.played_just_now');
  
  const now = Date.now();
  const diffMs = now - timestamp;
  
  if (diffMs < 0) return t('instance.played_just_now');
  
  const diffMinutes = Math.floor(diffMs / 60000);
  if (diffMinutes < 1) return t('instance.played_just_now');
  if (diffMinutes < 60) return t('instance.played_minutes_ago', { time: diffMinutes.toString() });
  
  const diffHours = Math.floor(diffMinutes / 60);
  if (diffHours < 24) return t('instance.played_hours_ago', { time: diffHours.toString() });
  
  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 30) return t('instance.played_days_ago', { time: diffDays.toString() });
  
  const diffMonths = Math.floor(diffDays / 30);
  if (diffMonths < 12) return t('instance.played_months_ago', { time: diffMonths.toString() });
  
  const diffYears = Math.floor(diffDays / 365);
  return t('instance.played_years_ago', { time: diffYears.toString() });
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

    <div class="tabs-wrapper">
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
          <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="2" y1="12" x2="22" y2="12"></line>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
          </svg>
          <span>{{ t('instance.worlds') }}</span>
        </div>
      </div>
      
      <div class="header-actions">
        <button v-if="activeTab === 0" class="btn-refresh btn-add-server" @click="openAddServer">
          <IconPlus class="add-server-icon" />
          <span>{{ t('instance.add_server') }}</span>
        </button>
        <button v-if="activeTab === 0" class="btn-refresh" @click="refreshServers">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 2v6h-6"></path>
            <path d="M21 13a9 9 0 1 1-3-7.7L21 8"></path>
          </svg>
          <span>{{ t('instance.refresh') }}</span>
        </button>
      </div>
    </div>

    <div class="tab-content">
      <div v-if="activeTab === 0 && servers.length === 0" class="empty-message">{{ t('instance.no_servers') }}</div>
      <div v-else-if="activeTab === 0" class="servers-list">
        <div class="server-card" v-for="server in servers" :key="server.ip">
          <img v-if="server.icon_base64" :src="`data:image/png;base64,${server.icon_base64}`" class="item-icon server-icon-img pixelated" alt="Server Icon" />
          <img v-else :src="`https://api.mcsrvstat.us/icon/${server.ip}`" @error="handleIconError" class="item-icon server-icon-img pixelated" alt="Server Icon" />
          <div class="server-card-content">
            <div class="server-details">
              <h4 class="item-name">{{ server.name }}</h4>
              <div class="server-ip">{{ server.ip }}</div>
            </div>
            <div class="server-motd-container">
              <div v-if="!server.motdHtml && server.loadingMotd" class="motd-loading-state">
                <svg class="motd-spinner" viewBox="0 0 50 50">
                  <circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="5"></circle>
                </svg>
                <span>{{ t('instance.loading') }}</span>
              </div>
              <div v-else-if="!server.online && !server.loadingMotd" class="server-motd is-error">{{ t('instance.cant_connect') }}</div>
              <div v-else class="server-motd" :class="{ 'is-loading': server.loadingMotd }" v-html="server.motdHtml"></div>
            </div>
            <div class="server-actions">
              <button class="btn-play-server" @click="handlePlayServer(server.ip)">
                <IconPlay class="play-icon-server" />
                {{ t('instance.play') || 'Play' }}
              </button>
              
              <div class="server-menu-wrapper" @click.stop>
                <button class="btn-server-menu" @click="toggleMenu(server.ip)">
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="1"></circle>
                    <circle cx="12" cy="5" r="1"></circle>
                    <circle cx="12" cy="19" r="1"></circle>
                  </svg>
                </button>
                
                <Transition name="dropdown">
                  <div v-if="activeMenu === server.ip" class="server-dropdown-menu">
                    <button class="dropdown-item" @click="copyAddress(server.ip)">
                      <svg class="dropdown-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                      </svg>
                      {{ t('instance.copy_address') }}
                    </button>
                    <button class="dropdown-item" @click="openEditServer(server)">
                      <svg class="dropdown-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M12 20h9"></path>
                        <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>
                      </svg>
                      {{ t('instance.edit_server') }}
                    </button>
                    <div class="dropdown-divider"></div>
                    <button class="dropdown-item item-danger" @click="removeServer(server.ip)">
                      <IconTrash class="dropdown-icon" />
                      {{ t('instance.remove_server') }}
                    </button>
                  </div>
                </Transition>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <div v-if="activeTab === 1 && worlds.length === 0" class="empty-message">{{ t('instance.no_worlds') }}</div>
      <div v-else-if="activeTab === 1" class="servers-list">
        <div class="server-card" v-for="world in worlds" :key="world.folder_name">
          <img v-if="world.icon_base64" :src="`data:image/png;base64,${world.icon_base64}`" class="item-icon server-icon-img pixelated" alt="World Icon" />
          <img v-else src="/server.png" class="item-icon server-icon-img pixelated" alt="Default World Icon" />
          <div class="server-card-content">
            <div class="server-details">
              <div class="world-title-row">
                <h4 class="item-name">{{ world.name }}</h4>
                <span class="world-folder-name">({{ world.folder_name }})</span>
              </div>
            </div>
            <div class="server-motd-container">
              <div class="server-motd is-world">{{ formatLastPlayed(world.last_played) }}</div>
            </div>
            <div class="server-actions" v-if="isWorldQuickPlaySupported">
              <button class="btn-play-server" @click="handlePlayWorld(world.folder_name)">
                <IconPlay class="play-icon-server" />
                {{ t('instance.play') || 'Play' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <Transition name="modal">
      <div v-if="isEditModalOpen" class="modal-backdrop" @click="closeEditModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2 class="modal-title">{{ isAddingServer ? t('instance.add_server_title') : t('instance.edit_server_title') }}</h2>
          <div class="close-control" @click="closeEditModal">
            <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
        </div>
        
        <div class="form-group">
          <label>{{ t('instance.server_name') }}</label>
          <input type="text" v-model="editServerForm.name" class="input-field" placeholder="Minecraft Server" />
        </div>
        
        <div class="form-group">
          <label>{{ t('instance.server_address') }}</label>
          <input type="text" v-model="editServerForm.ip" class="input-field" placeholder="example.visualclient.com.pl" />
        </div>
        
        <div class="form-group">
          <label>{{ t('instance.server_resource_packs') }}</label>
          <div class="custom-select-wrapper" @click="isRpSelectOpen = !isRpSelectOpen">
            <div class="input-field select-field custom-select-display">
              <span class="custom-select-text">
                {{ editServerForm.acceptTextures === 1 ? t('instance.rp_enabled') : (editServerForm.acceptTextures === 0 ? t('instance.rp_disabled') : t('instance.rp_prompt')) }}
              </span>
              <svg class="select-caret" :class="{ 'is-open': isRpSelectOpen }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </div>
            
            <Transition name="dropdown">
              <div v-if="isRpSelectOpen" class="custom-select-dropdown">
                <div class="custom-option" :class="{ 'is-selected': editServerForm.acceptTextures === 1 }" @click.stop="editServerForm.acceptTextures = 1; isRpSelectOpen = false">
                  {{ t('instance.rp_enabled') }}
                </div>
                <div class="custom-option" :class="{ 'is-selected': editServerForm.acceptTextures === null }" @click.stop="editServerForm.acceptTextures = null; isRpSelectOpen = false">
                  {{ t('instance.rp_prompt') }}
                </div>
                <div class="custom-option" :class="{ 'is-selected': editServerForm.acceptTextures === 0 }" @click.stop="editServerForm.acceptTextures = 0; isRpSelectOpen = false">
                  {{ t('instance.rp_disabled') }}
                </div>
              </div>
            </Transition>
          </div>
        </div>
        
        <div class="modal-actions">
          <button class="btn btn-secondary" @click="closeEditModal">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
            {{ t('instance.cancel') }}
          </button>
          <button class="btn btn-success" @click="saveServer">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path><polyline points="17 21 17 13 7 13 7 21"></polyline><polyline points="7 3 7 8 15 8"></polyline></svg>
            {{ isAddingServer ? t('instance.add_server') : t('instance.save') }}
          </button>
        </div>
      </div>
      </div>
    </Transition>

    <Transition name="modal">
      <div v-if="isDeleteModalOpen" class="modal-backdrop" @click="closeDeleteModal">
        <div class="modal-content" @click.stop>
          <div class="modal-header">
            <h2 class="modal-title">{{ t('instance.remove_server_title') || 'Remove Server' }}</h2>
            <div class="close-control" @click="closeDeleteModal">
              <svg width="16" height="16" viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
          </div>
          <p class="modal-text" style="color: var(--text-muted); font-size: 0.95rem;">{{ t('instance.remove_server_desc') || 'Are you sure you want to remove this server? This action cannot be undone.' }}</p>
          <div class="modal-actions">
            <button class="btn btn-secondary" @click="closeDeleteModal">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
              {{ t('instance.cancel') }}
            </button>
            <button class="btn btn-danger" @click="confirmRemoveServer">
              <IconTrash class="dropdown-icon" />
              {{ t('instance.remove_server') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

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

.tabs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.btn-add-server {
  background-color: transparent;
  border: 1px solid var(--border-line);
  border-radius: 12px;
  color: var(--text-main);
  padding: 8px 16px;
  height: auto;
  transition: background-color 0.2s, transform 0.1s cubic-bezier(0.4, 0.0, 0.2, 1);
  will-change: transform;
}

.btn-add-server:hover {
  background-color: color-mix(in srgb, var(--text-main) 10%, var(--surface-1));
  color: var(--text-main);
}

.btn-add-server:active {
  transform: scale(0.95);
}

.add-server-icon {
  width: 18px;
  height: 18px;
}

.instance-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0;
  padding-bottom: 32px;
  border-bottom: 1px solid var(--border-line);
}

.tabs-wrapper {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
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

.btn-refresh {
  background-color: transparent;
  border: none;
  border-radius: 999px;
  height: 38px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-muted);
  font-weight: 700;
  font-size: 0.95rem;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-top: 10px;
}

.btn-refresh:hover {
  background-color: color-mix(in srgb, var(--surface-2) 100%, white 8%);
  color: var(--text-main);
}

.btn-refresh:active {
  transform: scale(0.95);
}

.btn-refresh svg {
  width: 16px;
  height: 16px;
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

.btn-play-server {
  background-color: transparent;
  color: var(--text-main);
  border: 1px solid var(--border-line);
  padding: 8px 16px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 0.9rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: background-color 0.2s, transform 0.1s cubic-bezier(0.4, 0.0, 0.2, 1);
  will-change: transform;
  backface-visibility: hidden;
  line-height: 1;
}

.btn-play-server:hover {
  background-color: color-mix(in srgb, var(--text-main) 10%, transparent);
}

.btn-play-server:active {
  transform: scale(0.96);
}

.play-icon-server {
  width: 18px;
  height: 18px;
  fill: currentColor;
}

.server-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.server-menu-wrapper {
  position: relative;
}

.btn-server-menu {
  background: transparent;
  color: var(--text-muted);
  border: none;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  will-change: transform;
  backface-visibility: hidden;
}

.btn-server-menu:hover {
  background-color: color-mix(in srgb, var(--text-main) 10%, transparent);
  color: var(--text-main);
}

.btn-server-menu:active {
  transform: scale(0.95);
}

.server-dropdown-menu {
  position: absolute;
  top: calc(100% + 12px);
  right: 0;
  background-color: var(--bg-shell);
  border: 1px solid var(--border-line);
  border-radius: 12px;
  padding: 6px;
  min-width: 200px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.server-dropdown-menu::before {
  content: '';
  position: absolute;
  top: -6px;
  right: 14px;
  width: 12px;
  height: 12px;
  background-color: var(--bg-shell);
  border-top: 1px solid var(--border-line);
  border-left: 1px solid var(--border-line);
  transform: rotate(45deg);
  z-index: -1;
  border-bottom-right-radius: 2px;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease-out;
}
.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translate(10px, -10px);
}

.dropdown-item {
  background: transparent;
  border: none;
  color: var(--text-muted);
  padding: 10px 12px;
  border-radius: 10px;
  font-size: 0.9rem;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 10px;
  transition: background-color 0.2s, color 0.2s;
}

.dropdown-item:hover {
  background-color: var(--surface-hover);
  color: var(--text-main);
}

.dropdown-icon {
  width: 16px;
  height: 16px;
  stroke-width: 2.5;
  flex-shrink: 0;
}

.dropdown-divider {
  height: 1px;
  background-color: var(--border-line);
  margin: 4px 12px;
}

.item-danger {
  color: #ff6b6b;
}

.item-danger:hover {
  background-color: #ff6b6b !important;
  color: #000 !important;
}

.world-title-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.world-folder-name {
  font-size: 0.85rem;
  color: var(--text-muted);
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
  cursor: default;
}

.server-card-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  min-width: 0;
}

.server-details {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
  width: 180px;
  flex-shrink: 0;
}

.server-motd-container {
  flex: 1;
  display: flex;
  justify-content: flex-start;
}

.server-motd {
  font-size: 0.85rem;
  color: var(--text-muted);
  white-space: pre-wrap;
  line-height: 1.4;
  height: calc(1.4em * 2);
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  transition: opacity 0.2s ease;
}

.server-motd.is-error,
.server-motd.is-world {
  display: flex;
  align-items: center;
  -webkit-line-clamp: unset;
}

.server-motd.is-loading {
  opacity: 0.5;
}

.server-motd.is-error {
  color: var(--danger) !important;
}

.motd-loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 0.9rem;
  font-weight: 500;
  height: calc(1.4em * 2);
}

.motd-spinner {
  animation: motd-rotate 2s linear infinite;
  width: 18px;
  height: 18px;
}

.motd-spinner .path {
  stroke: var(--accent);
  stroke-linecap: round;
  animation: motd-dash 1.5s ease-in-out infinite;
}

@keyframes motd-rotate {
  100% { transform: rotate(360deg); }
}

@keyframes motd-dash {
  0% { stroke-dasharray: 1, 150; stroke-dashoffset: 0; }
  50% { stroke-dasharray: 90, 150; stroke-dashoffset: -35; }
  100% { stroke-dasharray: 90, 150; stroke-dashoffset: -124; }
}

.server-ip {
  font-size: 0.85rem;
  color: var(--text-muted);
  background-color: var(--surface-2);
  padding: 4px 10px;
  border-radius: 8px;
  font-family: monospace;
  white-space: nowrap;
}

.btn-play-server {
  height: 38px;
  padding: 0 20px;
  border-radius: 12px;
  background-color: color-mix(in srgb, var(--surface-2) 100%, white 6%);
  color: var(--text-muted);
  font-weight: 600;
  font-size: 0.95rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: none;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  flex-shrink: 0;
  will-change: transform;
}

.btn-play-server:hover {
  background-color: color-mix(in srgb, var(--surface-2) 100%, white 12%);
  color: color-mix(in srgb, var(--text-muted) 100%, white 15%);
}

.btn-play-server:active {
  transform: scale(0.96);
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

.server-icon-img {
  object-fit: cover;
  background-color: transparent;
}

.pixelated {
  image-rendering: pixelated;
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
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: var(--backdrop-dark);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 999;
}

.modal-content {
  width: 480px;
  background-color: var(--bg-shell);
  border-radius: 16px;
  border: 1px solid var(--border-line);
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 40px rgba(0,0,0,0.5);
  padding: 24px;
  gap: 20px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.modal-title {
  font-size: 1.3rem;
  font-weight: 700;
  color: var(--text-main);
  margin-bottom: 0;
}

.close-control {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  cursor: pointer;
  transition: background-color 0.2s, color 0.2s, transform 0.2s;
}

.close-control:hover {
  background-color: var(--danger);
  color: var(--color-white);
}

.close-control:active {
  transform: scale(0.85);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-weight: 600;
  font-size: 0.95rem;
  color: var(--text-main);
}

.input-field {
  width: 100%;
  height: 38px;
  background-color: var(--surface-2);
  border: 1px solid transparent;
  border-radius: 12px;
  padding: 0 16px;
  color: var(--text-main);
  font-family: inherit;
  font-size: 1rem;
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
  outline: none;
}

.input-field::placeholder {
  color: var(--text-muted);
  font-weight: 500;
}

.input-field:focus {
  background-color: var(--surface-2);
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.select-field {
  appearance: none;
}

.custom-select-wrapper {
  position: relative;
  width: 100%;
}
.custom-select-display {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  user-select: none;
}
.custom-select-text {
  font-weight: 500;
}
.select-caret {
  width: 16px;
  height: 16px;
  color: var(--text-muted);
  transition: transform 0.2s ease;
}
.select-caret.is-open {
  transform: rotate(180deg);
}
.custom-select-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  width: 100%;
  background-color: var(--bg-shell);
  border: 1px solid var(--border-line);
  border-radius: 12px;
  padding: 0;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
  z-index: 100;
  display: flex;
  flex-direction: column;
  gap: 0;
}
.custom-option {
  padding: 10px 12px;
  border-radius: 0;
  cursor: pointer;
  color: var(--text-muted);
  font-weight: 500;
  transition: all 0.2s ease;
}
.custom-option:first-child {
  border-top-left-radius: 12px;
  border-top-right-radius: 12px;
}
.custom-option:last-child {
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}
.custom-option:hover {
  background-color: var(--surface-hover);
  color: var(--text-main);
}
.custom-option.is-selected {
  background-color: color-mix(in srgb, var(--accent) 15%, transparent);
  color: var(--accent);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 10px;
}

.btn {
  height: 36px;
  padding: 0 16px;
  background-color: var(--surface-1);
  border: 4px solid var(--border-line);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-weight: 600;
  font-size: 1rem;
  color: var(--text-muted);
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  font-family: inherit;
  gap: 6px;
}

.btn:hover {
  background-color: var(--surface-hover);
}

.btn:active {
  transform: scale(0.92);
}

.btn-primary {
  border-color: var(--accent);
  color: var(--text-main);
  background-color: color-mix(in srgb, var(--accent) 15%, transparent);
}

.btn-primary:hover {
  background-color: color-mix(in srgb, var(--accent) 25%, transparent);
}

.btn-success {
  border-color: #10B981;
  color: #fff;
  background-color: color-mix(in srgb, #10B981 15%, transparent);
}

.btn-success:hover {
  background-color: color-mix(in srgb, #10B981 35%, transparent);
}

.btn-danger {
  border-color: var(--danger);
  color: var(--danger);
  background-color: color-mix(in srgb, var(--danger) 15%, transparent);
}

.btn-danger:hover {
  background-color: color-mix(in srgb, var(--danger) 35%, transparent);
  color: var(--color-white);
}

/* Modal Transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}
.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.95);
}
</style>
