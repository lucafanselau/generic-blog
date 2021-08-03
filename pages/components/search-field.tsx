import React, { FC } from 'react';

// className={'w-24 bg-gray-100 rounded-md'}

export const SearchField: FC = () => {
    return (
        <input
            type={'text'}
            className={
                'text-xs p-1 px-2 bg-gray-200 dark:bg-gray-500 border-none dark:text-white shadow-none rounded-md prose '
            }
            w:text={'dark:placeholder-white'}
            style={{ height: 24 }}
            placeholder={'Type to search...'}
        />
    );
};
