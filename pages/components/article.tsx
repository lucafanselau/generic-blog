import React, { FC } from 'react';

export const Article: FC = ({ children }) => {
    return <div className={'container p-8 prose'}>{children}</div>;
};
