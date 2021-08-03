import React, { FC } from 'react';
import { Header } from '../components/header';

export const PageLayout: FC = ({ children }) => {
    return (
        <div className={'dark:bg-gray-800 bg-white w-full transition-colors'}>
            <Header />
            <div className={'container mx-auto'}>{children}</div>
        </div>
    );
};
