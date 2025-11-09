<template>
  <div class="min-h-screen bg-gray-50">
    <Navigation v-if="hasDataset" :datasetTitle="datasetTitle" />
    
    <div class="p-8">
      <router-view />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import Navigation from './components/Navigation.vue'
import axios from 'axios'

export default defineComponent({
  components: { Navigation },
  setup() {
    const hasDataset = ref(false)
    const datasetTitle = ref('')
    const router = useRouter()

    async function checkDataset() {
      try {
        const resp = await axios.get('http://localhost:8081/api/v1/datasets/current/metadata')
        if (resp.data) {
          hasDataset.value = true
          datasetTitle.value = resp.data.filename || 'Dataset'
        }
      } catch (e) {
        hasDataset.value = false
      }
    }

    onMounted(() => {
      checkDataset()
      // Poll for dataset changes every 2 seconds
      setInterval(checkDataset, 2000)
    })

    return { hasDataset, datasetTitle }
  }
})
</script>
