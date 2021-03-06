import ReactDOM from 'react-dom';
import React from 'react';
import { getPage } from 'vite-plugin-ssr/client';
import { PageLayout } from './page-layout';
import 'virtual:windi.css';
import 'virtual:windi-devtools';
import '../themes/prism-one-dark.css';
import '../themes/katex.css';
// import 'highlight.js/styles/atom-one-dark.css';

hydrate();

async function hydrate() {
    const pageContext = await getPage();
    const { Page, pageProps } = pageContext;

    if (window.matchMedia('(prefers-color-scheme: dark)').matches) document.documentElement.classList.add('dark');
    else document.documentElement.classList.add('light');

    ReactDOM.hydrate(
        <PageLayout>
            <Page {...pageProps} />
        </PageLayout>,
        document.getElementById('page-view'),
    );
}
