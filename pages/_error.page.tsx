import React, { useCallback, useEffect, MouseEvent, FC, useRef, useState } from 'react';
import init, { Game } from 'rust-404';

export { Page };

const renderLoop = (g: Game, last: number) => {
    let now = window.performance.now();
    g.update((now - last) / 1000.0);
    g.render();
    requestAnimationFrame(() => renderLoop(g, now));
};

const Page404: FC = () => {
    const [game, setGame] = useState<Game | undefined>(undefined);

    useEffect(() => {
        (async () => {
            await init();
        })();
    }, []);

    useEffect(() => {
        return () => {
            if (game !== undefined) game.free();
        };
    }, [game]);

    const onClick = useCallback(
        async (event: MouseEvent<HTMLCanvasElement>) => {
            // lock the pointer
            event.currentTarget.requestPointerLock();

            if (game === undefined) {
                const g = await Game.new();

                // start render loop
                let startup = window.performance.now();
                console.log('new');
                setGame(g);
                renderLoop(g, startup);
            }
        },
        [game],
    );

    return (
        <div className={'flex flex-col space-y-2 max-w-full items-center prose '}>
            <h2>404 Page Not Found</h2>
            <p>This page could not be found. You can however enjoy urself playing a little</p>
            <div
                className={
                    'rounded-xl shadow-dark-50 shadow-md from-green-200 to-blue-100 bg-gradient-to-l p-4 dark:bg-gray-800'
                }
            >
                <canvas id={'canvas'} width={600} height={400} className={''} onClick={onClick} />
            </div>
        </div>
    );
};

function Page({ is404 }: { is404: boolean }) {
    if (is404) {
        return <Page404 />;
    } else {
        return (
            <>
                <h1>500 Internal Server Error</h1>
                Something went wrong.
            </>
        );
    }
}
