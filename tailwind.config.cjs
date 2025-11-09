/** Minimal Tailwind config for this repo.
 * Vite/PostCSS will pick this up; no need to run the CLI to generate it.
 */
module.exports = {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}'
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
