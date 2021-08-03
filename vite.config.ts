import reactRefresh from '@vitejs/plugin-react-refresh';
import { UserConfig } from 'vite';

import ssr from 'vite-plugin-ssr/plugin';
import WindiCSS from 'vite-plugin-windicss';
import mdx from 'vite-plugin-mdx';
import svgr from '@svgr/rollup';

const config: UserConfig = {
    plugins: [
        reactRefresh(),
        WindiCSS(),
        ssr(),
        mdx(),
        // NOTE(luca): Apparently vite's types think are not compatible to rollup plugins, even though they work
        // @ts-ignore
        svgr(),
    ],
};

export default config;
