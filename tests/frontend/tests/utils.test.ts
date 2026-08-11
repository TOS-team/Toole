// je teste les utilitaires purs du frontend (formatage taille, extension,
// visuel associé à un fichier)

import { describe, expect, it } from "vitest";
import { extOf, fileVisual, formatSize } from "../../../desktop-app/ui/src/utils";

describe("formatSize", () => {
  it("affiche les octets bruts en dessous de 1000", () => {
    expect(formatSize(0)).toBe("0 o");
    expect(formatSize(512)).toBe("512 o");
    expect(formatSize(999)).toBe("999 o");
  });

  it("enchaîne Ko, Mo, Go en unités décimales (×1000)", () => {
    expect(formatSize(1000)).toBe("1.0 Ko");
    expect(formatSize(1_500)).toBe("1.5 Ko");
    expect(formatSize(2_500_000)).toBe("2.5 Mo");
    expect(formatSize(3_500_000_000)).toBe("3.5 Go");
  });

  it("gère les valeurs intermédiaires sans arrondi trompeur", () => {
    expect(formatSize(1_999_950)).toBe("2.0 Mo");
    expect(formatSize(999_999)).toBe("1000.0 Ko");
  });
});

describe("extOf", () => {
  it("renvoie l'extension en minuscules sans point", () => {
    expect(extOf("photo.JPG")).toBe("jpg");
    expect(extOf("Archives.Rar")).toBe("rar");
    expect(extOf("rapport.final.md")).toBe("md");
  });

  it("renvoie vide pour un nom sans extension", () => {
    expect(extOf("makefile")).toBe("");
    expect(extOf("fichier.")).toBe("");
  });

  it("ne traite pas un point initial comme une extension", () => {
    expect(extOf(".bashrc")).toBe("");
  });
});

describe("fileVisual", () => {
  it("identifie les images avec miniature", () => {
    expect(fileVisual("photo.png")).toEqual({ icon: "image", thumb: true });
    expect(fileVisual("a.svg")).toEqual({ icon: "image", thumb: true });
  });

  it("classe vidéos, musiques et archives", () => {
    expect(fileVisual("film.mkv").icon).toBe("movie");
    expect(fileVisual("chanson.mp3").icon).toBe("music");
    expect(fileVisual("archive.zip").icon).toBe("folder-zip");
  });

  it("classe code et documents", () => {
    expect(fileVisual("main.rs").icon).toBe("code");
    expect(fileVisual("note.txt").icon).toBe("document");
  });

  it("retombe sur l'icône générique par défaut", () => {
    expect(fileVisual("inconnu.xyz")).toEqual({ icon: "file", thumb: false });
  });
});