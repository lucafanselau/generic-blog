import reactRefresh from "@vitejs/plugin-react-refresh";
import ssr from "vite-plugin-ssr/plugin";
import { UserConfig } from "vite";
import WindiCSS from "vite-plugin-windicss";

const config: UserConfig = {
  plugins: [reactRefresh(), WindiCSS(), ssr()],
};

export default config;
