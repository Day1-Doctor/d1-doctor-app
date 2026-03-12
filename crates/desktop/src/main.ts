import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import i18n, { initLocale } from './plugins/i18n'
import './shared/styles/tokens.css'
import './shared/styles/animations.css'
import './shared/styles/a11y.css'

const app = createApp(App)
app.use(createPinia())
app.use(i18n)
app.mount('#app')

// Detect system locale asynchronously after mount
initLocale()
