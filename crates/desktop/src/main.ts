import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './shared/styles/tokens.css'
import './shared/styles/animations.css'

const app = createApp(App)
app.use(createPinia())
app.mount('#app')
