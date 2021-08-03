import ReactDOMServer from 'react-dom/server';
import React from 'react';
import { PageLayout } from './page-layout';
import { html } from 'vite-plugin-ssr';
import { PageContext } from './types';

export { render };
export { passToClient };

// See https://vite-plugin-ssr.com/data-fetching
const passToClient = ['pageProps'];

function render(pageContext: PageContext) {
    const { Page, pageProps } = pageContext;
    const pageHtml = ReactDOMServer.renderToString(
        <PageLayout>
            <Page {...pageProps} />
        </PageLayout>,
    );

    // See https://vite-plugin-ssr.com/html-head
    const { documentProps } = pageContext;
    const title = (documentProps && documentProps.title) || 'Vite SSR app';
    const desc = (documentProps && documentProps.description) || 'App using Vite + vite-plugin-ssr';

    return html`<!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="description" content="${desc}" />
                <title>${title}</title>
            </head>
            <body>
                <div id="page-view">${html.dangerouslySkipEscape(pageHtml)}</div>
            </body>
        </html>`;
}
