// stub de listen() pour les tests : je remplace l'abonnement Tauri par une
// table de handlers que les tests peuvent déclencher manuellement
//
// émettre un événement "tool://..." = appeler `emit` avec le nom exact
// enregistré par le store

type Handler = (event: { payload: unknown }) => void;

const handlers = new Map<string, Handler>();

export async function listen<T>(
  event: string,
  handler: (event: { payload: T }) => void,
): Promise<() => void> {
  handlers.set(event, handler as Handler);
  return () => {
    handlers.delete(event);
  };
}

// je simule l'arrivée d'un événement venant du processus Rust
export function emit(event: string, payload: unknown) {
  const h = handlers.get(event);
  if (!h) {
    throw new Error(`aucun listener enregistré pour ${event}`);
  }
  h({ payload });
}

export function resetEmit() {
  handlers.clear();
}