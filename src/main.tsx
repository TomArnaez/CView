import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.tsx";
import "./index.css";
import "./styles.css";
import { MantineProvider } from "@mantine/core";
import { ContextMenuProvider } from "mantine-contextmenu";
import "@mantine/core/styles.css";
import "mantine-contextmenu/styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <MantineProvider>
    <ContextMenuProvider>
      <React.StrictMode>
        <App />
      </React.StrictMode>
    </ContextMenuProvider>
  </MantineProvider>
);
