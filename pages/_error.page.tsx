import React, { useCallback, useEffect, MouseEvent } from 'react';
import init, { Game } from 'rust-404';

export { Page };

const renderLoop = (g: Game, last: number) => {
    let now = window.performance.now();
    g.update((now - last) / 1000.0);
    g.render();
    requestAnimationFrame(() => renderLoop(g, now));
};

function Page({ is404 }: { is404: boolean }) {
    useEffect(() => {
        if (is404) {
            (async () => {
                await init();
            })();
        }
    }, [is404]);

    const onClick = useCallback(async (event: MouseEvent<HTMLCanvasElement>) => {
        // lock the pointer
        event.currentTarget.requestPointerLock();

        const g = Game.new();
        g.init();

        // start render loop
        let startup = window.performance.now();
        renderLoop(g, startup);
    }, []);

    if (is404) {
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
    } else {
        return (
            <>
                <h1>500 Internal Server Error</h1>
                Something went wrong.
            </>
        );
    }
}
