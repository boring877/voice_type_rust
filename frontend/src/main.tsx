import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import HudApp from "./HudApp";
import "./app.css";

const currentWindowLabel = getCurrentWindow().label;
const hudWindow = currentWindowLabel === "hud";
const RootComponent = hudWindow ? HudApp : App;

document.documentElement.classList.toggle("window-hud", hudWindow);
document.body.classList.toggle("window-hud", hudWindow);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RootComponent />
  </React.StrictMode>,
);
