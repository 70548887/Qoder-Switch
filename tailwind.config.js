export default {
  content: ['./index.html', './src/**/*.{vue,js,ts}'],
  theme: {
    extend: {
      colors: {
        dark: { 900: '#0f0f1a', 800: '#1a1a2e', 700: '#16213e', 600: '#1e2a4a' },
        accent: { DEFAULT: '#4361ee', hover: '#3651de' },
      }
    }
  }
}
