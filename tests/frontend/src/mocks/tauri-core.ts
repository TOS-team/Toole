// stub de invoke() pour les tests : je remplace le pont Tauri par une table
// de réponses pilotable par les tests
//
// chaque test peut enregistrer une réponse pour une commande précise puis
// vérifier la liste des commandes réellement appelées

type Command = string;

const responses = new Map<Command, unknown>();
const calls: Command[] = [];

// je fournis une réponse pour une commande donnée (appelée par le code UI)
export function mockReply(cmd: Command, value: unknown) {
  responses.set(cmd, value);
}

export function calledCommands(): Command[] {
  return [...calls];
}

export function resetMock() {
  responses.clear();
  calls.length = 0;
}

export async function invoke<T>(cmd: Command, _args?: Record<string, unknown>): Promise<T> {
  calls.push(cmd);
  if (!responses.has(cmd)) {
    throw new Error(`commande non mockee: ${cmd}`);
  }
  return responses.get(cmd) as T;
}