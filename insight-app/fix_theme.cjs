const fs = require('fs');
const path = require('path');

const cssVars = `
:root {
  --bg: #F5F0EB;
  --bg-solid: #F5F0EB;
  --text-primary: #2C2420;
  --text-secondary: #6B5E54;
  --text-muted: #A89A8E;
  --text-dim: #D1C5B8;
  --surface-02: rgba(0,0,0,0.02);
  --surface-03: rgba(0,0,0,0.03);
  --surface-04: rgba(0,0,0,0.04);
  --surface-05: rgba(0,0,0,0.05);
  --surface-06: rgba(0,0,0,0.06);
  --surface-08: rgba(0,0,0,0.08);
  --surface-10: rgba(0,0,0,0.1);
  --surface-12: rgba(0,0,0,0.12);
  --surface-15: rgba(0,0,0,0.15);
  --surface-25: rgba(0,0,0,0.25);
}

:root.night {
  --bg: #1c1917;
  --bg-solid: #1C1917;
  --text-primary: #e8e0d8;
  --text-secondary: #9e958c;
  --text-muted: #6e6760;
  --text-dim: #5a544e;
  --surface-02: rgba(255,255,255,0.02);
  --surface-03: rgba(255,255,255,0.03);
  --surface-04: rgba(255,255,255,0.04);
  --surface-05: rgba(255,255,255,0.05);
  --surface-06: rgba(255,255,255,0.06);
  --surface-08: rgba(255,255,255,0.08);
  --surface-10: rgba(255,255,255,0.1);
  --surface-12: rgba(255,255,255,0.12);
  --surface-15: rgba(255,255,255,0.15);
  --surface-25: rgba(255,255,255,0.25);
}
`;

const appVuePath = path.join(__dirname, 'src/dashboard/App.vue');
let appVue = fs.readFileSync(appVuePath, 'utf8');

// Replace colors
const replacements = [
  ['#1c1917', 'var(--bg)'],
  ['#e8e0d8', 'var(--text-primary)'],
  ['#d4cdc5', 'var(--text-primary)'], // Similar enough
  ['#9e958c', 'var(--text-secondary)'],
  ['#6e6760', 'var(--text-muted)'],
  ['#7a746e', 'var(--text-muted)'], // Similar enough
  ['#5a544e', 'var(--text-dim)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.02\)/g, 'var(--surface-02)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.03\)/g, 'var(--surface-03)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.04\)/g, 'var(--surface-04)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.05\)/g, 'var(--surface-05)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.06\)/g, 'var(--surface-06)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.08\)/g, 'var(--surface-08)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.1\)/g, 'var(--surface-10)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.12\)/g, 'var(--surface-12)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.15\)/g, 'var(--surface-15)'],
  [/rgba\(255,\s*255,\s*255,\s*0\.25\)/g, 'var(--surface-25)']
];

for (const [from, to] of replacements) {
  if (typeof from === 'string') {
    appVue = appVue.split(from).join(to);
  } else {
    appVue = appVue.replace(from, to);
  }
}

// Add state and theme synchronization
if (!appVue.includes("const theme = ref")) {
  appVue = appVue.replace(
    /const locale = getLocale\(\)/,
    `const theme = ref<'day' | 'night'>('day')
const locale = getLocale()

async function loadTheme() {
  try {
    const saved = await invoke<string>('get_theme')
    if (saved === 'day' || saved === 'night') {
      theme.value = saved
      if (theme.value === 'night') {
        document.documentElement.classList.add('night')
      } else {
        document.documentElement.classList.remove('night')
      }
    }
  } catch (e) {
    console.warn('loadTheme failed', e)
  }
}
`
  );

  appVue = appVue.replace(
    /onMounted\(async \(\) => \{/,
    `import { listen } from '@tauri-apps/api/event'

onMounted(async () => {
  await loadTheme()
  listen('theme-changed', (event: any) => {
    theme.value = event.payload
    if (theme.value === 'night') {
      document.documentElement.classList.add('night')
    } else {
      document.documentElement.classList.remove('night')
    }
  })
`
  );
}

fs.writeFileSync(appVuePath, appVue);

const styleCssPath = path.join(__dirname, 'src/dashboard/style.css');
let styleCss = fs.readFileSync(styleCssPath, 'utf8');

if (!styleCss.includes(':root')) {
  styleCss = cssVars + '\n' + styleCss;
}

styleCss = styleCss.replace(/#1c1917/g, 'var(--bg)').replace(/#e8e0d8/g, 'var(--text-primary)');

fs.writeFileSync(styleCssPath, styleCss);

console.log('Theme updated successfully.');
