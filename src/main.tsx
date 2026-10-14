import ReactDOM from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./i18n";
import "@xterm/xterm/css/xterm.css";
import "./styles.css";

async function start() {
  // Nur im Entwicklungsmodus UND im normalen Browser: Backend simulieren (siehe devmock.ts).
  // In der fertigen App ist `import.meta.env.DEV` false – der Code fällt komplett weg.
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    (await import("./devmock")).install();
  }
  // Bewusst ohne StrictMode: der würde im Dev-Modus zwei Shells starten.
  ReactDOM.createRoot(document.getElementById("root")!).render(
    <I18nProvider>
      <App />
    </I18nProvider>,
  );
}
void start();
