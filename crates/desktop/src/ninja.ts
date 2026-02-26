import { createApp } from 'vue'
import { createPinia } from 'pinia'
import NinjaApp from './NinjaApp.vue'
import './shared/styles/tokens.css'
import './shared/styles/animations.css'

const app = createApp(NinjaApp)
app.use(createPinia())
app.mount('#ninja-app')
