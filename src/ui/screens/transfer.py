import os
from tkinter import filedialog
from typing import List

import customtkinter as ctk
from controller.controller import Controller
from theme import FONT_FAMILY, THEME, msgbox


class TransferScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master, fg_color="transparent")
        self.controller = controller
        self.selected_files: List[str] = []
        self._last_signature = None
        self._layout_mode = "horizontal"
        self._is_sending = False

        self._recipient_values = ["Everyone (Broadcast)"]
        self.recipient_var = ctk.StringVar(value=self._recipient_values[0])

        self.body = ctk.CTkScrollableFrame(
            self,
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"],
        )
        self.body.pack(fill="both", expand=True)

        self._build_ui()
        self._apply_layout()
        self.bind("<Configure>", self._on_configure)
        self.refresh_recipients(force=True)
        self.refresh_selected_files()

    def _build_ui(self):
        self.left_panel = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )

        self.files_header = ctk.CTkFrame(self.left_panel, fg_color="transparent")
        self.files_header.pack(fill="x", padx=16, pady=(16, 8))

        self.files_title = ctk.CTkLabel(
            self.files_header,
            text="Fichiers a envoyer",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        )

        self.file_actions = ctk.CTkFrame(self.files_header, fg_color="transparent")

        ctk.CTkButton(
            self.file_actions,
            text="Ajouter Fichier",
            width=110,
            height=30,
            font=ctk.CTkFont(family=FONT_FAMILY, size=11, weight="bold"),
            fg_color=THEME["bg_subcard"],
            text_color=THEME["text_main"],
            hover_color=THEME["border"],
            command=self.select_files,
        ).pack(side="left", padx=4)

        meta_card = ctk.CTkFrame(
            self.left_panel, fg_color=THEME["bg_subcard"], corner_radius=8
        )
        meta_card.pack(fill="x", padx=16, pady=4)

        self.files_count = ctk.CTkLabel(
            meta_card,
            text="0 element",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_main"],
        )
        self.files_count.pack(side="left", padx=12, pady=6)

        self.total_size_label = ctk.CTkLabel(
            meta_card,
            text="Taille totale: 0 B",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12),
            text_color=THEME["text_muted"],
        )
        self.total_size_label.pack(side="right", padx=12, pady=6)

        self.files_list = ctk.CTkScrollableFrame(
            self.left_panel,
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"],
        )
        self.files_list.pack(fill="both", expand=True, padx=8, pady=(4, 8))

        self.clear_btn = ctk.CTkButton(
            self.left_panel,
            text="Vider la selection",
            height=36,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            fg_color=THEME["danger"],
            hover_color=THEME["danger_hover"],
            command=self.clear_selection,
        )
        self.clear_btn.pack(fill="x", padx=16, pady=16)

        self.right_panel = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )

        ctk.CTkLabel(
            self.right_panel,
            text="Configuration d'envoi",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(anchor="w", padx=16, pady=(16, 8))

        status_box = ctk.CTkFrame(
            self.right_panel, fg_color=THEME["bg_subcard"], corner_radius=8
        )
        status_box.pack(fill="x", padx=16, pady=4)

        ctk.CTkLabel(
            status_box,
            text="STATUT",
            font=ctk.CTkFont(family=FONT_FAMILY, size=9, weight="bold"),
            text_color=THEME["text_dim"],
        ).pack(anchor="w", padx=12, pady=(8, 2))

        self.connection_state = ctk.CTkLabel(
            status_box,
            text="Non connecte",
            font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
            text_color=THEME["danger"],
            wraplength=360,
            justify="left",
        )
        self.connection_state.pack(anchor="w", padx=12, pady=(0, 10))

        recipient_card = ctk.CTkFrame(
            self.right_panel, fg_color=THEME["bg_subcard"], corner_radius=8
        )
        recipient_card.pack(fill="x", padx=16, pady=8)

        ctk.CTkLabel(
            recipient_card,
            text="DESTINATAIRE",
            font=ctk.CTkFont(family=FONT_FAMILY, size=9, weight="bold"),
            text_color=THEME["text_dim"],
        ).pack(anchor="w", padx=12, pady=(8, 4))

        self.recipient_menu = ctk.CTkOptionMenu(
            recipient_card,
            variable=self.recipient_var,
            values=self._recipient_values,
            fg_color=THEME["bg_card"],
            button_color=THEME["border"],
            button_hover_color=THEME["bg_subcard"],
            dropdown_fg_color=THEME["bg_subcard"],
            dropdown_hover_color=THEME["border"],
            font=ctk.CTkFont(family=FONT_FAMILY, size=12),
            height=36,
            command=lambda _: self._update_send_state(),
        )
        self.recipient_menu.pack(fill="x", padx=12, pady=(0, 10))

        progress_card = ctk.CTkFrame(
            self.right_panel, fg_color=THEME["bg_subcard"], corner_radius=8
        )
        progress_card.pack(fill="x", padx=16, pady=4)

        self.progress_label = ctk.CTkLabel(
            progress_card,
            text="Pret pour l'envoi",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_muted"],
            wraplength=360,
            justify="left",
        )
        self.progress_label.pack(anchor="w", padx=12, pady=(8, 4))

        self.progress = ctk.CTkProgressBar(
            progress_card, fg_color=THEME["bg_card"], progress_color=THEME["primary"]
        )
        self.progress.set(0)
        self.progress.pack(fill="x", padx=12, pady=(0, 12))

        self.send_btn = ctk.CTkButton(
            self.right_panel,
            text="Envoyer",
            height=42,
            font=ctk.CTkFont(family=FONT_FAMILY, size=14, weight="bold"),
            fg_color=THEME["success"],
            hover_color=THEME["success_hover"],
            command=self.send_files,
            state="disabled",
        )
        self.send_btn.pack(fill="x", padx=16, pady=(16, 8))

        ctk.CTkButton(
            self.right_panel,
            text="Rafraichir les destinataires",
            height=34,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            fg_color=THEME["bg_subcard"],
            hover_color=THEME["border"],
            command=lambda: self.refresh_recipients(force=True),
        ).pack(fill="x", padx=16, pady=(0, 16))

    def _on_configure(self, event):
        if event.widget == self:
            width = event.width
            new_mode = "vertical" if width < 820 else "horizontal"
            if new_mode != self._layout_mode:
                self._layout_mode = new_mode
                self._apply_layout()

    def _apply_layout(self):
        self.files_title.pack_forget()
        self.file_actions.pack_forget()
        if self._layout_mode == "vertical":
            self.files_title.pack(anchor="w", fill="x")
            self.file_actions.pack(anchor="w", fill="x", pady=(8, 0))
        else:
            self.files_title.pack(side="left")
            self.file_actions.pack(side="right")

        self.left_panel.grid_forget()
        self.right_panel.grid_forget()
        for i in range(2):
            self.body.grid_columnconfigure(i, weight=0, minsize=0)
            self.body.grid_rowconfigure(i, weight=0, minsize=0)

        if self._layout_mode == "vertical":
            self.body.grid_columnconfigure(0, weight=1, minsize=0)
            self.body.grid_rowconfigure(0, weight=1, minsize=220)
            self.body.grid_rowconfigure(1, weight=0, minsize=0)
            self.left_panel.grid(row=0, column=0, sticky="nsew", padx=0, pady=(0, 8))
            self.right_panel.grid(row=1, column=0, sticky="ew", padx=0, pady=(8, 0))
        else:
            self.body.grid_columnconfigure(0, weight=2, minsize=320)
            self.body.grid_columnconfigure(1, weight=1, minsize=280)
            self.body.grid_rowconfigure(0, weight=1, minsize=0)
            self.left_panel.grid(row=0, column=0, sticky="nsew", padx=(0, 8), pady=4)
            self.right_panel.grid(row=0, column=1, sticky="nsew", padx=(8, 0), pady=4)

    def _format_size(self, size: int) -> str:
        value = float(size)
        for unit in ["B", "KB", "MB", "GB", "TB"]:
            if value < 1024.0:
                return f"{value:.1f} {unit}"
            value /= 1024.0
        return f"{value:.1f} PB"

    def _file_info(self, path: str):
        is_dir = os.path.isdir(path)
        size = 0
        if not is_dir and os.path.exists(path):
            size = os.path.getsize(path)
        ext = os.path.splitext(path)[1].lower() if not is_dir else "<dossier>"
        return is_dir, size, ext

    def _peers_signature(self, peers):
        return tuple(
            (p.get("id"), p.get("ip"), p.get("tcp_port"), p.get("role")) for p in peers
        )

    def _update_send_state(self):
        if self._is_sending:
            self.send_btn.configure(state="disabled", fg_color=THEME["primary"])
            self.clear_btn.configure(state="disabled")
            return

        self.clear_btn.configure(state="normal")
        can_transfer, reason = self.controller.can_transfer(self.recipient_var.get())
        allowed = can_transfer and len(self.selected_files) > 0
        self.send_btn.configure(
            state="normal" if allowed else "disabled",
            fg_color=THEME["success"] if allowed else THEME["border"],
        )
        if can_transfer:
            self.connection_state.configure(
                text="Actif (pret pour transfert)", text_color=THEME["success"]
            )
        else:
            self.connection_state.configure(text=reason, text_color=THEME["danger"])

    def refresh_recipients(self, force: bool = False):
        snapshot, peers = self.controller.get_runtime_view()
        sig = (snapshot.get("state") if snapshot else -1, self._peers_signature(peers))
        if not force and sig == self._last_signature:
            self._update_send_state()
            return
        self._last_signature = sig

        values = self.controller.recipient_labels()
        self._recipient_values = values
        self.recipient_menu.configure(values=values)
        if self.recipient_var.get() not in values:
            self.recipient_var.set(values[0])

        self._update_send_state()

    def select_files(self):
        files = filedialog.askopenfilenames(title="Selectionner des fichiers")
        if files:
            for path in files:
                if path not in self.selected_files:
                    self.selected_files.append(path)
            self.refresh_selected_files()

    def refresh_selected_files(self):
        for widget in self.files_list.winfo_children():
            widget.destroy()

        count = len(self.selected_files)
        self.files_count.configure(text=f"{count} element{'s' if count > 1 else ''}")

        total = 0
        for path in self.selected_files:
            is_dir, size, _ = self._file_info(path)
            if not is_dir:
                total += size
        self.total_size_label.configure(
            text=f"Taille totale: {self._format_size(total)}"
        )

        if count == 0:
            ctk.CTkLabel(
                self.files_list,
                text="Aucun fichier selectionne",
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_muted"],
            ).pack(pady=40)
            self._update_send_state()
            return

        for path in self.selected_files:
            is_dir, size, ext = self._file_info(path)
            card = ctk.CTkFrame(
                self.files_list,
                fg_color=THEME["bg_subcard"],
                border_width=1,
                border_color=THEME["border"],
                corner_radius=10,
            )
            card.pack(fill="x", padx=4, pady=4)

            name = os.path.basename(path) or path
            kind = (
                "Dossier"
                if is_dir
                else f"Fichier {ext.upper() if ext else '<SANS EXT>'}"
            )
            size_text = "--" if is_dir else self._format_size(size)

            info = ctk.CTkFrame(card, fg_color="transparent")
            info.pack(side="left", fill="both", expand=True, padx=12, pady=10)

            ctk.CTkLabel(
                info,
                text=name,
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_main"],
                anchor="w",
                justify="left",
            ).pack(anchor="w", fill="x")

            ctk.CTkLabel(
                info,
                text=f"{kind} - {size_text}",
                font=ctk.CTkFont(family=FONT_FAMILY, size=11),
                text_color=THEME["text_muted"],
                anchor="w",
                justify="left",
            ).pack(anchor="w", fill="x", pady=(2, 0))

            ctk.CTkButton(
                card,
                text="Retirer",
                width=70,
                height=28,
                font=ctk.CTkFont(family=FONT_FAMILY, size=11, weight="bold"),
                fg_color="transparent",
                text_color=THEME["danger"],
                hover_color=THEME["bg_card"],
                command=lambda p=path: self.remove_file(p),
            ).pack(side="right", padx=10)

        self._update_send_state()

    def clear_selection(self):
        self.selected_files.clear()
        self.refresh_selected_files()

    def remove_file(self, path: str):
        if path in self.selected_files:
            self.selected_files.remove(path)
        self.refresh_selected_files()

    # ==================== START TRANSFER ====================

    def send_files(self):
        if not self.selected_files:
            msgbox(
                title="Erreur", message="Aucun fichier selectionne.", icon="warning"
            )
            return

        can_transfer, reason = self.controller.can_transfer(self.recipient_var.get())
        if not can_transfer:
            msgbox(title="Connexion requise", message=reason, icon="warning")
            return

        self._is_sending = True
        self._update_send_state()
        self.progress.configure(progress_color=THEME["primary"])
        self.progress.set(0)
        self.progress_label.configure(text="Progression: 0%")

        import queue
        import threading

        q = queue.Queue()

        def transfer_worker():
            files_to_send = list(self.selected_files)
            total = len(files_to_send)
            sent = 0
            failed = []

            for idx, path in enumerate(files_to_send, start=1):
                ok, msg = self.controller.send_single_file(
                    path, self.recipient_var.get()
                )
                if ok:
                    sent += 1
                else:
                    failed.append(f"{os.path.basename(path)}: {msg}")
                q.put(("file_done", os.path.basename(path), ok, msg))
                q.put(("progress", idx, total))

            q.put(("done", sent, total, failed))

        threading.Thread(target=transfer_worker, daemon=True).start()
        self._check_transfer_queue(q)

    def _update_bridge_progress(self):
        status = self.controller.get_transfer_status()
        total = int(status.get("total", 0) or 0)
        sent = int(status.get("sent", 0) or 0)
        filename = status.get("filename") or "fichier"
        if total > 0 and int(status.get("status", 0)) == 1:
            ratio = max(0.0, min(1.0, sent / total))
            self.progress.set(ratio)
            self.progress_label.configure(
                text=f"{filename}: {int(ratio * 100)}% ({self._format_size(sent)} / {self._format_size(total)})"
            )

    def _check_transfer_queue(self, q):
        import queue

        self._update_bridge_progress()
        try:
            while True:
                msg = q.get_nowait()
                if msg[0] == "progress":
                    _, idx, total = msg
                    if not self.controller.get_transfer_status().get("active"):
                        ratio = idx / total
                        self.progress.set(ratio)
                        self.progress_label.configure(
                            text=f"Progression: {int(ratio * 100)}% ({idx}/{total})"
                        )
                elif msg[0] == "done":
                    _, sent, total, failed = msg
                    self.progress.set(1.0)
                    self.progress.configure(
                        progress_color=THEME["success"]
                        if not failed
                        else THEME["warning"]
                    )
                    self.progress_label.configure(text=f"Termine ({sent}/{total})")
                    self._is_sending = False
                    self._update_send_state()

                    if failed:
                        msgbox(
                            title="Transfert partiel",
                            message=f"{sent}/{total} envoye(s).\n\n"
                            + "\n".join(failed[:8]),
                            icon="warning",
                        )
                    else:
                        msgbox(
                            title="Succes",
                            message=f"{sent}/{total} fichier(s) envoye(s).",
                            icon="check",
                        )
                    return
        except queue.Empty:
            self.after(50, lambda: self._check_transfer_queue(q))
