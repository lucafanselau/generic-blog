import reactRefresh from '@vitejs/plugin-react-refresh';
import { defineConfig, UserConfig } from 'vite';

import ssr from 'vite-plugin-ssr/plugin';
import WindiCSS from 'vite-plugin-windicss';
import mdx, { MdxOptions } from 'vite-plugin-mdx';
import svgr from '@svgr/rollup';
import ViteRsw from 'vite-plugin-rsw';
import rehypePrism from '@mapbox/rehype-prism';

export default defineConfig(async (env): Promise<UserConfig> => {
    const remarkMath = (await import('remark-math')).default;
    const rehypeKatex = (await import('rehype-katex')).default;

    const mdxOptions: MdxOptions = {
        remarkPlugins: [remarkMath],
        rehypePlugins: [rehypePrism, rehypeKatex],
    };

    return {
        plugins: [
            reactRefresh(),
            WindiCSS(),
            ssr(),
            mdx(mdxOptions),
            // NOTE(luca): Apparently vite's types think are not compatible to rollup plugins, even though they work
            // @ts-ignore
            svgr(),
            ViteRsw({
                root: 'packages',
                crates: ['rust-404'],
            }),
        ],
    };
});
