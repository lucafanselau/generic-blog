import { defineConfig } from 'vite-plugin-windicss';
import typography from 'windicss/plugin/typography';
import forms from 'windicss/plugin/forms';
import animations from '@windicss/plugin-animations';

export default defineConfig({
    darkMode: 'class',
    extract: {
        include: ['pages/**/*.{vue,html,tsx,ts}'],
    },
    attributify: {
        prefix: 'w:',
    },
    plugins: [typography({ dark: true }), forms, animations()],
});
