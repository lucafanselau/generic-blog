import { motion, Variants } from 'framer-motion';
import React, { FC, useCallback, useEffect, useState } from 'react';

const grey = '#27272a';
const pathVariants: Variants = {
    on: {
        translateX: 36,
        rotate: -90,
        fill: '#FFF',
        transition: { type: 'tween' },
    },
    off: {
        translateX: 0,
        rotate: 0,
        fill: grey,
        transition: { type: 'tween' },
    },
};

export const DarkModeToggle: FC = () => {
    const [toggled, setToggled] = useState(false);

    const handleClick = useCallback(() => {
        setToggled((t) => !t);
    }, []);

    useEffect(() => {
        if (toggled) {
            // -> Dark Mode
            document.documentElement.classList.remove('light');
            document.documentElement.classList.add('dark');
        } else {
            // -> Light Mode
            document.documentElement.classList.remove('dark');
            document.documentElement.classList.add('light');
        }
    }, [toggled]);

    return (
        <svg
            width="42"
            height="24"
            viewBox="0 0 84 48"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            onClick={handleClick}
        >
            <motion.rect
                x="2"
                y="2"
                width="80"
                height="44"
                rx="22"
                animate={{ stroke: toggled ? '#FFF' : grey }}
                strokeWidth="4"
            />
            <motion.path
                fill-rule="evenodd"
                clip-rule="evenodd"
                variants={pathVariants}
                animate={toggled ? 'on' : 'off'}
                d="M37.1138 34.8188L34.8188 37.1138C31.8794 39.5415 28.11 41 24 41C14.6112 41 7 33.3888 7 24C7 16.7401 11.5508 10.5432 17.9553 8.10605L39.894 30.0447C39.2289 31.7924 38.2839 33.402 37.1138 34.8188Z"
            />
            <motion.g
                key={'sun'}
                animate={{ opacity: !toggled ? 1 : 0 }}
                transition={{ type: 'tween', duration: 0.2 }}
                stroke={grey}
                strokeWidth={'2'}
                strokeLinecap={'round'}
                strokeLinejoin={'round'}
                fill={grey}
            >
                <path d="M60 13V15" />
                <path d="M60 33V35" />
                <path d="M52.22 16.22L53.64 17.64" />
                <path d="M66.36 30.36L67.78 31.78" />
                <path d="M49 24H51" />
                <path d="M69 24H71" />
                <path d="M52.22 31.78L53.64 30.36" />
                <path d="M66.36 17.64L67.78 16.22" />
                <circle cx="60" cy="24" r="6" stroke={'none'} />
            </motion.g>
            <motion.path
                animate={{ opacity: toggled ? 1 : 0 }}
                transition={{ type: 'tween', duration: 0.2 }}
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M34.264 30.2204C32.939 30.7241 31.5017 31 30 31C23.3726 31 18 25.6274 18 19C18 16.7227 18.6344 14.5936 19.736 12.7796C15.2137 14.4992 12 18.8743 12 24C12 30.6274 17.3726 36 24 36C28.3501 36 32.1596 33.6853 34.264 30.2204Z"
                fill="white"
            />
        </svg>
    );
};
