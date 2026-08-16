// config WebdriverIO pour piloter l'app Toolé avec tauri-driver (WebDriver
// natif du webview WebKitGTK). Je lance tauri-driver moi-même dans onPrepare
// et je l'arrête dans onComplete : le test ne dépend d'aucun process externe.
import type { Options } from "@wdio/types";
import { spawn, type ChildProcess } from "node:child_process";

let driver: ChildProcess | null = null;

// j'attends que tauri-driver réponde sur son endpoint /status (sans délai
// fixe : sous charge la liaison du port peut prendre du temps)
async function waitForDriver(
  host: string,
  port: number,
  timeoutMs = 60000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://${host}:${port}/status`);
      if (res.ok) return;
    } catch {
      // pas encore prêt : je réessaie
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`tauri-driver ne répond pas sur ${host}:${port}`);
}

export const config: Options.Testrunner = {
  runner: "local",
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: { transpileOnly: true, project: "./tsconfig.json" },
  },
  specs: ["./tests/**/*.e2e.ts"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      browserName: "wry",
      // WebKitWebDriver ne supporte pas le protocole BiDi : je force le
      // protocole WebDriver classique (sinon WDIO ajoute webSocketUrl et la
      // création de session échoue)
      "wdio:enforceWebDriverClassic": true,
      // chemin du binaire de l'app compilée (workspace racine du repo) :
      // « cargo build -p app » l'a produit dans target/debug/app
      "tauri:options": {
        application: "../../target/debug/app",
      },
    },
  ],
  hostname: "127.0.0.1",
  port: 4444,
  path: "/",
  logLevel: "info",
  waitforTimeout: 15000,
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,
  framework: "mocha",
  mochaOpts: { ui: "bdd", timeout: 60000 },
  reporters: ["spec"],

  onPrepare: async () => {
    driver = spawn("tauri-driver", ["--port", "4444"], { stdio: "inherit" });
    await waitForDriver("127.0.0.1", 4444);
  },
  onComplete: () => {
    driver?.kill();
  },
};