// test e2e de smoke test : je lance la vraie app (binaire compilé + webview
// WebKitGTK) et je vérifie que la chaîne complète fonctionne :
//   Vue (rendu) → invoke → commands.rs → toole_core (get_device_id, get_peers)
// C'est le seul niveau de test qui traverse le pont IPC réel.
//
// Note : WebKitWebDriver (WebDriver GTK) ne renvoie pas le texte des éléments
// via getElementText et ne supporte pas l'endpoint « element click » : je lis
// donc le texte et je clique via executeScript, qui touche le vrai DOM.
import { browser } from "@wdio/globals";

const POLL_MS = 200;

// je lis le textContent d'un élément, null s'il n'existe pas
async function textOf(selector: string): Promise<string | null> {
  const res = await browser.execute((sel) => {
    const el = document.querySelector(sel);
    return el ? (el.textContent ?? "").trim() : null;
  }, selector);
  return res ?? null;
}

// j'attends qu'un élément existe et affiche le texte attendu (le montage Vue
// est asynchrone, je ne suppose pas de délai fixe)
async function waitForText(
  selector: string,
  expected: string,
  timeoutMs = 20000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const text = await textOf(selector);
    if (text === expected) return;
    await browser.pause(POLL_MS);
  }
  throw new Error(
    `"${selector}" ne vaut pas "${expected}" (valeur: ${JSON.stringify(await textOf(selector))})`,
  );
}

// je clique via JS : la vraie app réagit au clic natif de HTMLElement.click()
async function clickJs(selector: string): Promise<void> {
  const done = await browser.execute((sel) => {
    const el = document.querySelector<HTMLElement>(sel);
    if (!el) return false;
    el.click();
    return true;
  }, selector);
  if (!done) throw new Error(`élément introuvable pour cliquer : ${selector}`);
}

describe("Toolé e2e — front → backend", () => {
  it("démarre et rend la page d'accueil", async () => {
    await waitForText("h1", "Bienvenue");
    // la zone de dépôt est aussi rendue (contient « Déposer des fichiers »)
    const drop = await textOf("main");
    expect(drop !== null && drop.includes("Déposer des fichiers")).toBe(true);
  });

  it("affiche l'identité de la machine venue du backend (IPC réel)", async () => {
    // le champ est rempli par invoke("get_device_id") → commands.rs →
    // toole_core::utils::device_id() : s'il est non vide, tout le pont marche
    const id = await textOf("span.text-primary.font-semibold");
    expect(id !== null && id.length > 0).toBe(true);
  });

  it("démarre la découverte : le panneau des appareils est présent", async () => {
    // le montage appelle invoke("start_discovery") puis poll get_peers toutes
    // les 2 s : le panneau doit afficher l'en-tête avec le compteur
    await waitForText("h3", "Appareils (0)");
  });

  it("navigue vers la page des transferts", async () => {
    await clickJs('button[title="Transferts"]');
    await waitForText("h1", "Transfert");
  });

  it("navigue vers la page des paramètres", async () => {
    await clickJs('button[title="Paramètres"]');
    await waitForText("h1", "Paramètres");
  });
});