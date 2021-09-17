import React, { FC, useCallback } from 'react';
// import hljs from 'highlight.js';
// import 'highlight.js/styles/github.css';

export const CodeBlock: FC<any> = (props) => {
    return (
        <React.Fragment>
            <pre>
                <code {...props}>{...props.children}</code>
            </pre>
            <div className={'flex justify-end'}>
                <button
                    className={
                        'bg-white text-grey-800 dark:bg-grey-800 dark:text-white rounded-md p-1 animate-bounce focus:outline-none'
                    }
                >
                    Copy
                </button>
            </div>
        </React.Fragment>
    );
};
