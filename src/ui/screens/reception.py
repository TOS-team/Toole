import os
from tkinter import filedialog

import customtkinter as ctk
from controller.controller import Controller
from theme import FONT_FAMILY, THEME, msgbox


class ReceptionScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master, fg_color="transparent")
        self.controller = controller

        self.body = ctk.CTkScrollableFrame(
            self,
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"],
        )
        self.body.pack(fill="both", expand=True)

        self._build_ui()
        self.refresh_received()

    def _build_ui(self):
        config_card = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )
        config_card.pack(fill="x", pady=(0, 12))

        ctk.CTkLabel(
            config_card,
            text="Dossier de reception",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(anchor="w", padx=16, pady=(16, 8))

        self.dir_label = ctk.CTkLabel(
            config_card,
            text=self.controller.receive_dir,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12),
            text_color=THEME["text_muted"],
            anchor="w",
            justify="left",
            wraplength=720,
        )
        self.dir_label.pack(fill="x", padx=16, pady=(0, 10))

        actions = ctk.CTkFrame(config_card, fg_color="transparent")
        actions.pack(fill="x", padx=16, pady=(0, 16))

        ctk.CTkButton(
            actions,
            text="Choisir un dossier",
            height=36,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            fg_color=THEME["primary"],
            hover_color=THEME["primary_hover"],
            command=self.choose_receive_dir,
        ).pack(side="left", fill="x", expand=True, padx=(0, 6))

        ctk.CTkButton(
            actions,
            text="Rafraichir",
            height=36,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            fg_color=THEME["bg_subcard"],
            hover_color=THEME["border"],
            command=self.refresh_received,
        ).pack(side="right", fill="x", expand=True, padx=(6, 0))

        files_card = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )
        files_card.pack(fill="both", expand=True)

        ctk.CTkLabel(
            files_card,
            text="Fichiers recus",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(anchor="w", padx=16, pady=(16, 8))

        self.files_list = ctk.CTkScrollableFrame(
            files_card,
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"],
        )
        self.files_list.pack(fill="both", expand=True, padx=8, pady=(0, 8))

    def _format_size(self, size: int) -> str:
        value = float(size)
        for unit in ["B", "KB", "MB", "GB", "TB"]:
            if value < 1024.0:
                return f"{value:.1f} {unit}"
            value /= 1024.0
        return f"{value:.1f} PB"

    def choose_receive_dir(self):
        folder = filedialog.askdirectory(title="Choisir le dossier de reception")
        if not folder:
            return
        ok, msg = self.controller.set_receive_dir(folder)
        msgbox(
            title="Reception" if ok else "Erreur",
            message=msg,
            icon="check" if ok else "warning",
        )
        self.refresh_received()

    def refresh_received(self):
        self.dir_label.configure(text=self.controller.receive_dir)
        for widget in self.files_list.winfo_children():
            widget.destroy()

        files = self.controller.get_received_files()
        if not files:
            ctk.CTkLabel(
                self.files_list,
                text="Aucun fichier recu pour le moment",
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_muted"],
            ).pack(anchor="w", padx=12, pady=12)
            return

        for item in files:
            card = ctk.CTkFrame(
                self.files_list,
                fg_color=THEME["bg_subcard"],
                border_width=1,
                border_color=THEME["border"],
                corner_radius=10,
            )
            card.pack(fill="x", padx=4, pady=4)

            name = item.get("name", "?")
            path = item.get("path", "")
            size = int(item.get("size", 0))
            when = item.get("time", "--:--:--")

            ctk.CTkLabel(
                card,
                text=name,
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_main"],
                anchor="w",
                justify="left",
            ).pack(anchor="w", fill="x", padx=12, pady=(10, 2))

            ctk.CTkLabel(
                card,
                text=f"{self._format_size(size)} - recu a {when}\n{path}",
                font=ctk.CTkFont(family=FONT_FAMILY, size=11),
                text_color=THEME["text_muted"],
                anchor="w",
                justify="left",
                wraplength=720,
            ).pack(anchor="w", fill="x", padx=12, pady=(0, 10))
