<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { t } from '../../../composables/useI18n';

interface VersionManifestResponse {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: Array<{
    id: string;
    type: string;
    url: string;
    time: string;
    releaseTime: string;
  }>;
}

const props = defineProps<{
  modelValue: string;
  loader: string;
}>();

const emit = defineEmits(['update:modelValue', 'update:url']);

const versions = ref<{ id: string, label: string }[]>([]);
const versionUrls = ref<Record<string, string>>({});
const isFetchingVersions = ref(true);

const versionSearch = ref(props.modelValue);
const isUserTyping = ref(false);

const isDropdownOpen = ref(false);
const dropdownDirection = ref('down');
const dropdownListRef = ref<HTMLElement | null>(null);
const containerRef = ref<HTMLElement | null>(null);

const filteredVersions = computed(() => {
  if (!isUserTyping.value) {
    return versions.value;
  }
  return versions.value.filter(ver => 
    ver.label.toLowerCase().includes(versionSearch.value.toLowerCase()) || ver.id.toLowerCase().includes(versionSearch.value.toLowerCase())
  );
});

const onSearchInput = () => {
  isUserTyping.value = true;
  emit('update:modelValue', versionSearch.value);
  emit('update:url', versionUrls.value[versionSearch.value] || '');
};

const closeDropdown = (e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.custom-select-container')) {
    isDropdownOpen.value = false;
  }
};

import { invoke } from '@tauri-apps/api/core';

const fetchVersions = async () => {
  isFetchingVersions.value = true;
  versions.value = [];
  try {
    if (props.loader === 'forge') {
      const responseText = await invoke<string>('fetch_forge_versions');
      const data = JSON.parse(responseText);
      const forgeMcVersions = new Set<string>();
      if (data.promos) {
         for (const key of Object.keys(data.promos)) {
            const mcVer = key.replace('-latest', '').replace('-recommended', '');
            forgeMcVersions.add(mcVer);
         }
      }
      
      const res = await fetch('https://launchermeta.mojang.com/mc/game/version_manifest.json');
      const manifest: VersionManifestResponse = await res.json();
      const relVersions = manifest.versions.filter(v => v.type === 'release' && forgeMcVersions.has(v.id));
      versions.value = relVersions.map(v => ({ id: v.id, label: v.id }));
      relVersions.forEach(v => {
        versionUrls.value[v.id] = v.url;
      });
    } else {
      const res = await fetch('https://launchermeta.mojang.com/mc/game/version_manifest.json');
      const data: VersionManifestResponse = await res.json();
      const relVersions = data.versions.filter(v => v.type === 'release');
      versions.value = relVersions.map(v => ({ id: v.id, label: v.id }));
      relVersions.forEach(v => {
        versionUrls.value[v.id] = v.url;
      });
    }
    
    if (versions.value.length > 0) {
      versionSearch.value = versions.value[0].label;
      emit('update:modelValue', versions.value[0].id);
      emit('update:url', versionUrls.value[versions.value[0].id] || '');
    }
  } catch (e) {
    console.error('Failed to fetch versions', e);
  } finally {
    isFetchingVersions.value = false;
  }
};

import { watch } from 'vue';
watch(() => props.loader, fetchVersions);

onMounted(async () => {
  document.addEventListener('click', closeDropdown);
  await fetchVersions();
});

onUnmounted(() => {
  document.removeEventListener('click', closeDropdown);
});

const openDropdown = async (e: Event) => {
  if (!isDropdownOpen.value) {
    isDropdownOpen.value = true;
    isUserTyping.value = false;
    await nextTick();
    if (e.target && (e.target as HTMLInputElement).select) {
      (e.target as HTMLInputElement).select();
    }
    if (dropdownListRef.value && e.target) {
      const rect = (e.target as HTMLElement).getBoundingClientRect();
      const dropdownHeight = dropdownListRef.value.offsetHeight || 250;
      const windowHeight = window.innerHeight;
      if (rect.bottom + dropdownHeight > windowHeight - 20) {
        dropdownDirection.value = 'up';
      } else {
        dropdownDirection.value = 'down';
      }
      
      setTimeout(() => {
        if (dropdownListRef.value) {
          const list = dropdownListRef.value.querySelector('.versions-list');
          const selectedEl = list?.querySelector('.version-card.selected') as HTMLElement;
          if (list && selectedEl) {
            list.scrollTop = selectedEl.offsetTop - (list.clientHeight / 2) + (selectedEl.clientHeight / 2);
          }
        }
      }, 10);
    }
  }
};

const selectVersion = (v: { id: string, label: string }) => {
  versionSearch.value = v.label;
  isDropdownOpen.value = false;
  emit('update:modelValue', v.id);
  emit('update:url', versionUrls.value[v.id] || '');
};
</script>

<template>
  <div class="custom-select-container" ref="containerRef">
    <div style="position: relative;">
      <input 
        type="text" 
        v-model="versionSearch" 
        placeholder="Search version..." 
        class="form-input version-input"
        :class="{ open: isDropdownOpen }"
        @focus="openDropdown"
        @click="openDropdown"
        @input="onSearchInput"
        :disabled="isFetchingVersions"
      />
      <svg class="trigger-icon" :class="{ open: isDropdownOpen }" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="6 9 12 15 18 9"></polyline>
      </svg>
    </div>
    
    <Transition name="dropdown-fade">
      <div 
        v-if="isDropdownOpen" 
        class="custom-select-dropdown" 
        :class="dropdownDirection"
        ref="dropdownListRef"
      >
        <div class="versions-list">
          <div v-if="isFetchingVersions" class="loading-text">{{ t('create_instance.no_versions') }}</div>
          <div v-else-if="filteredVersions.length === 0" class="loading-text">{{ t('create_instance.no_versions_found') }}</div>
          <template v-else>
            <div 
              v-for="v in filteredVersions" 
              :key="v.id" 
              class="version-card"
              :class="{ selected: versionSearch === v.label }"
              @click="selectVersion(v)"
            >
              <span class="version-name">{{ v.label }}</span>
            </div>
          </template>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.form-input {
  width: 100%;
  height: 44px;
  background-color: var(--surface-1);
  border: none;
  border-radius: 12px;
  padding: 0 16px;
  color: var(--text-main);
  font-family: inherit;
  font-size: 1rem;
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
  outline: none;
}

.form-input::placeholder {
  color: var(--text-muted);
}

.form-input:focus, .form-input.open {
  background-color: var(--surface-hover);
  box-shadow: 0 0 0 4px var(--accent);
}

.custom-select-container {
  position: relative;
}

.trigger-icon {
  position: absolute;
  right: 16px;
  top: 14px;
  pointer-events: none;
  color: var(--text-muted);
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.trigger-icon.open {
  transform: rotate(180deg);
}

.custom-select-dropdown {
  position: absolute;
  left: 0;
  right: 0;
  background-color: var(--bg-shell);
  border: 1px solid var(--border-line);
  border-radius: 12px;
  z-index: 100;
  box-shadow: 0 10px 40px var(--backdrop-dark);
  display: flex;
  flex-direction: column;
}

.custom-select-dropdown.down {
  top: calc(100% + 8px);
}

.custom-select-dropdown.up {
  bottom: calc(100% + 8px);
}

.versions-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 240px;
  overflow-y: auto;
  padding: 8px;
}

.versions-list::-webkit-scrollbar {
  width: 6px;
}
.versions-list::-webkit-scrollbar-track {
  background: transparent;
  margin: 8px 0;
}
.versions-list::-webkit-scrollbar-thumb {
  background: var(--border-line);
  border-radius: 6px;
}
.versions-list::-webkit-scrollbar-thumb:hover {
  background: var(--text-muted);
}

.version-card {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  background-color: transparent;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.version-card:hover {
  background-color: var(--surface-hover);
}

.version-card.selected {
  background-color: var(--surface-hover);
  color: var(--text-main);
}

.version-name {
  font-weight: 600;
  color: var(--text-main);
}

.loading-text {
  text-align: center;
  padding: 24px 16px;
  font-size: 1.05rem;
  color: var(--text-muted);
}

/* Dropdown Animation */
.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity 0.2s, transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
}

.dropdown-fade-enter-from.down {
  transform: translateY(-10px);
}
.dropdown-fade-leave-to.down {
  transform: translateY(-10px);
}

.dropdown-fade-enter-from.up {
  transform: translateY(10px);
}
.dropdown-fade-leave-to.up {
  transform: translateY(10px);
}
</style>
