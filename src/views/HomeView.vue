<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { t } from '../composables/useI18n';

interface ModrinthProject {
  slug: string;
  title: string;
  description: string;
  icon_url: string;
  gallery: string[];
}

const shaders = ref<ModrinthProject[]>([]);
const mods = ref<ModrinthProject[]>([]);
const modpacks = ref<ModrinthProject[]>([]);
const isLoadingShaders = ref(true);
const isLoadingMods = ref(true);
const isLoadingModpacks = ref(true);

onMounted(() => {
  fetch('https://api.modrinth.com/v2/search?limit=5&facets=[[%22project_type:shader%22]]')
    .then(res => res.json())
    .then(data => { shaders.value = data.hits || []; })
    .catch(error => console.error('Failed to fetch shaders', error))
    .finally(() => { isLoadingShaders.value = false; });
    
  fetch('https://api.modrinth.com/v2/search?limit=5&facets=[[%22project_type:mod%22]]')
    .then(res => res.json())
    .then(data => { mods.value = data.hits || []; })
    .catch(error => console.error('Failed to fetch mods', error))
    .finally(() => { isLoadingMods.value = false; });
    
  fetch('https://api.modrinth.com/v2/search?limit=5&facets=[[%22project_type:modpack%22]]')
    .then(res => res.json())
    .then(data => { modpacks.value = data.hits || []; })
    .catch(error => console.error('Failed to fetch modpacks', error))
    .finally(() => { isLoadingModpacks.value = false; });
});
</script>

<template>
  <div class="home-view">
    <h1 class="welcome-title">{{ t('home.welcome') }}</h1>
    
    <div class="discover-section">
      <h2 class="section-title">{{ t('home.discover_shaders') }}</h2>
      
      <div v-if="isLoadingShaders" class="cards-grid">
        <div class="skeleton-card" v-for="i in 5" :key="'s-'+i"></div>
      </div>
      
      <div v-else class="cards-grid">
        <div class="card" v-for="item in shaders" :key="item.slug">
          <div class="card-top">
            <img :src="item.gallery && item.gallery.length > 0 ? item.gallery[0] : item.icon_url" alt="" class="card-bg" />
          </div>
          <div class="card-bottom">
            <div class="card-header">
              <img :src="item.icon_url" alt="" class="card-icon" />
              <h3 class="card-title">{{ item.title }}</h3>
            </div>
            <p class="card-description">{{ item.description }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="discover-section mods-section">
      <h2 class="section-title">{{ t('home.discover_mods') }}</h2>
      
      <div v-if="isLoadingMods" class="cards-grid">
        <div class="skeleton-card" v-for="i in 5" :key="'m-'+i"></div>
      </div>
      
      <div v-else class="cards-grid">
        <div class="card" v-for="item in mods" :key="item.slug">
          <div class="card-top">
            <img :src="item.gallery && item.gallery.length > 0 ? item.gallery[0] : item.icon_url" alt="" class="card-bg" />
          </div>
          <div class="card-bottom">
            <div class="card-header">
              <img :src="item.icon_url" alt="" class="card-icon" />
              <h3 class="card-title">{{ item.title }}</h3>
            </div>
            <p class="card-description">{{ item.description }}</p>
          </div>
        </div>
      </div>
    </div>
    
    <div class="discover-section mods-section">
      <h2 class="section-title">{{ t('home.discover_modpacks') }}</h2>
      
      <div v-if="isLoadingModpacks" class="cards-grid">
        <div class="skeleton-card" v-for="i in 5" :key="'mp-'+i"></div>
      </div>
      
      <div v-else class="cards-grid">
        <div class="card" v-for="item in modpacks" :key="item.slug">
          <div class="card-top">
            <img :src="item.gallery && item.gallery.length > 0 ? item.gallery[0] : item.icon_url" alt="" class="card-bg" />
          </div>
          <div class="card-bottom">
            <div class="card-header">
              <img :src="item.icon_url" alt="" class="card-icon" />
              <h3 class="card-title">{{ item.title }}</h3>
            </div>
            <p class="card-description">{{ item.description }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  padding: 0;
  padding-bottom: 32px;
}

.welcome-title {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--text-main);
  margin-top: 0;
  margin-bottom: 32px;
}

.discover-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.mods-section {
  margin-top: 32px;
}

.section-title {
  color: var(--text-muted);
  font-size: 1.1rem;
  font-weight: 600;
  margin: 0;
}

.skeleton-card {
  height: 280px;
  background-color: var(--surface-1);
  border-radius: 12px;
  animation: pulse 1.5s infinite ease-in-out;
}

.skeleton-card:nth-child(n+4) {
  display: none;
}

@keyframes pulse {
  0% { opacity: 0.5; }
  50% { opacity: 1; }
  100% { opacity: 0.5; }
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.card {
  height: 280px;
  background-color: var(--bg-shell);
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), background-color 0.2s, box-shadow 0.2s;
  cursor: pointer;
  border: 1px solid var(--border-line);
}

.card:nth-child(n+4) {
  display: none;
}

.card:hover {
  background-color: var(--surface-hover);
  box-shadow: 0 8px 24px var(--overlay-medium);
}

.card-top {
  height: 140px;
  width: 100%;
  overflow: hidden;
  background-color: var(--surface-1);
}

.card-bg {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.3s ease;
}

.card:hover .card-bg {
  transform: scale(1.05);
}

.card-bottom {
  height: 140px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  object-fit: cover;
  background-color: var(--surface-1);
}

.card-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-main);
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-description {
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--text-muted);
  line-height: 1.4;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

@media (min-width: 1280px) {
  .cards-grid {
    grid-template-columns: repeat(4, 1fr);
  }
  .card:nth-child(4), .skeleton-card:nth-child(4) {
    display: flex;
  }
}

@media (min-width: 1550px) {
  .cards-grid {
    grid-template-columns: repeat(5, 1fr);
  }
  .card:nth-child(5), .skeleton-card:nth-child(5) {
    display: flex;
  }
}
</style>
