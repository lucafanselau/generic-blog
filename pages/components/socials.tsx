import React, { FC } from 'react';
import { ReactComponent as GithubIcon } from './github.svg';

export const Socials: FC = () => {
    return (
        <React.Fragment>
            <button
                className={'bg-gray-800 dark:bg-white rounded-md p-1 hover:animate-pulse focus:outline-none'}
                style={{ width: 24, height: 24 }}
            >
                <GithubIcon width={'auto'} height={'auto'} className={'fill-white dark:fill-gray-800'} />
            </button>
        </React.Fragment>
    );
};
