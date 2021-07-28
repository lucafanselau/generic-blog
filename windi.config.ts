import { defineConfig } from "vite-plugin-windicss";

export default defineConfig({
    extract: {
        include: ['pages/**/*.{vue,html,tsx,ts}']
    },
    attributify: {
        prefix: "w:"
    }
});