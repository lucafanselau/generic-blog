import { defineConfig } from 'vite-plugin-windicss';
import typography from 'windicss/plugin/typography';

export default defineConfig({
    darkMode: 'class',
    extract: {
        include: ['pages/**/*.{vue,html,tsx,ts}'],
    },
    attributify: {
        prefix: 'w:',
    },
    plugins: [typography({ dark: true })],
});
