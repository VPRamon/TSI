import { createApp } from 'vue'
import AppNew from './AppNew.vue'
import router from './router'
import './styles.css'

createApp(AppNew)
  .use(router)
  .mount('#app')
