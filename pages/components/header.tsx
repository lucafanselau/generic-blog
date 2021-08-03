import React, { FC } from 'react';
import { DarkModeToggle } from './dark-mode';
import { SearchField } from './search-field';
import { Socials } from './socials';

export const Header: FC = () => {
    return (
        <div className={'flex justify-between border-b border-gray-800 dark:border-white items-center p-2'}>
            <p className={'font-mono text-lg text-gray-800 dark:text-white leading-none'}>Generic Blog</p>
            <div className={'flex flex-row-reverse items-stretch   space-x-2 space-x-reverse'}>
                <Socials />
                <DarkModeToggle />
                <SearchField />
            </div>
        </div>
    );
};
