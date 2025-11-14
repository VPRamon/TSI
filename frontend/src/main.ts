import { createApp } from 'vue'
import AppNew from './AppNew.vue'
import router from './router'

// Import global styles
import '@/shared/styles/main.css'

// Create and mount the app
const app = createApp(AppNew)

app.use(router)

app.mount('#app')
