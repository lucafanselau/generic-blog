import React, { useEffect } from 'react';
import init, { Game } from 'rust-404';

export { Page };

const renderLoop = (g: Game) => {
    g.render();
    requestAnimationFrame(() => renderLoop(g));
};

function Page({ is404 }: { is404: boolean }) {
    useEffect(() => {
        if (is404) {
            (async () => {
                await init();
                const g = Game.new();
                g.init();

                renderLoop(g);
            })();
        }
    }, [is404]);

    if (is404) {
        return (
            <div className={'flex flex-col space-y-2 max-w-full items-center prose '}>
                <h2>404 Page Not Found</h2>
                <p>This page could not be found. You can however enjoy urself playing a little</p>
                <canvas id={'canvas'} width={600} height={400} className={''} />
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
