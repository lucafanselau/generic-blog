import React, { FC } from 'react';

export const Article: FC = ({ children }) => {
    return <div className={'container prose'}>{children}</div>;
};
