import { createApp } from 'vue'
import { createPinia } from 'pinia'
import NinjaApp from './NinjaApp.vue'
import i18n, { initLocale } from './plugins/i18n'
import './shared/styles/tokens.css'
import './shared/styles/animations.css'
import './shared/styles/a11y.css'

const app = createApp(NinjaApp)
app.use(createPinia())
app.use(i18n)
app.mount('#ninja-app')

// Detect system locale asynchronously after mount
initLocale()
