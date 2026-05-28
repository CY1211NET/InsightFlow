import { createApp } from 'vue'
import App from './App.vue'

try {
  const app = createApp(App)

  // Global error handler to surface errors visually
  app.config.errorHandler = (err, _vm, info) => {
    console.error('[Note Vue Error]', err, info)
    showError(`Vue Runtime Error: ${err}\n\nInfo: ${info}`)
  }

  app.mount('#app')
  console.log('[Note] Vue app mounted successfully')
} catch (e: any) {
  console.error('[Note] Failed to mount Vue app:', e)
  showError(`Mount Error: ${e?.message || e}`)
}

function showError(msg: string) {
  const el = document.getElementById('app')
  if (el) {
    // Don't replace existing content if Vue already rendered something
    const errorDiv = document.createElement('div')
    errorDiv.className = 'mount-error'
    errorDiv.textContent = msg
    el.appendChild(errorDiv)
  }
}
