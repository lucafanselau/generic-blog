import React, { FC, useEffect } from 'react';
import { Header } from '../components/header';

export const PageLayout: FC = ({ children }) => {
    return (
        <div className={'dark:bg-gray-800 bg-white w-full transition-colors'}>
            <Header />
            <div className={'container mx-auto p-8'}>{children}</div>
        </div>
    );
};
