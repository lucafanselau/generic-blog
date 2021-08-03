import React, { FC } from 'react';
import { DarkModeToggle } from './dark-mode';

export const Header: FC = () => {
    return (
        <div className={'flex flex-row-reverse p-1'}>
            <div className={'space-x-2 space-x-reverse'}>
                <DarkModeToggle />
            </div>
        </div>
    );
};
