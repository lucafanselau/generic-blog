import React, { FC } from 'react';
import { MDXProvider } from '@mdx-js/react';
import { CodeBlock } from './code-block';

export const Article: FC = ({ children }) => {
    const components = {
        pre: (props: any) => <div {...props} />,
        code: CodeBlock,
    };
    return (
        <MDXProvider components={components}>
            <div className={'container prose'}>{children}</div>
        </MDXProvider>
    );
};
