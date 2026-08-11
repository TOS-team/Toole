// je re-exporte invoke() depuis le pont Tauri : centraliser cet import permet
// aux tests de le remplacer facilement (voir tests/frontend/src/mocks/)
import { invoke } from "@tauri-apps/api/core";

export { invoke };
