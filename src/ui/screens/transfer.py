import os
import time
from tkinter import filedialog
from typing import List

import customtkinter as ctk
from controller.controller import Controller
from CTkMessagebox import CTkMessagebox

# Hello la BOP, on inline le theme directement ici pour eviter theme.py
THEME = {
    "bg_dark": "#0B0F19",       # Deep space navy background
    "bg_card": "#152238",       # Sleek card container background
    "bg_subcard": "#1E2D4A",    # Inner sub-card background
    "border": "#243B5A",        # Thin sharp borders
    "primary": "#6366F1",       # Indigo accent
    "primary_hover": "#4F46E5",
    "success": "#10B981",       # Teal accent
    "success_hover": "#059669",
    "warning": "#F59E0B",       # Amber warning
    "danger": "#EF4444",        # Rose danger
    "danger_hover": "#DC2626",
    "text_main": "#F3F4F6",     # Almost white main text
    "text_muted": "#9CA3AF",    # Light grey muted text
    "text_dim": "#6B7280"       # Dark grey dim text
}

FONT_FAMILY = "Segoe UI"


class TransferScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master, fg_color="transparent")
        self.controller = controller
        self.selected_files: List[str] = []
        self._last_signature = None
        self._layout_mode = "horizontal"
        self._is_sending = False
        
        # History log list
        self.history: List[dict] = []

        self._build_ui()
        self._apply_layout()
        self.bind("<Configure>", self._on_configure)
        self.refresh_recipients(force=True)
        self.refresh_selected_files()
        self.refresh_history_ui()

    def _build_ui(self):
        # 1. Left Panel (Files Selection)
        self.left_panel = ctk.CTkFrame(
            self, 
            fg_color=THEME["bg_card"], 
            border_width=1, 
            border_color=THEME["border"],
            corner_radius=12
        )

        top_left = ctk.CTkFrame(self.left_panel, fg_color="transparent")
        top_left.pack(fill="x", padx=16, pady=(16, 8))

        ctk.CTkLabel(
            top_left, 
            text="Fichiers a envoyer", 
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"]
        ).pack(side="left")

        # Action buttons
        btn_box = ctk.CTkFrame(top_left, fg_color="transparent")
        btn_box.pack(side="right")

        ctk.CTkButton(
            btn_box, 
            text="Ajouter Fichier", 
            width=100, 
            height=30, 
            font=ctk.CTkFont(family=FONT_FAMILY, size=11, weight="bold"),
            fg_color=THEME["bg_subcard"],
            text_color=THEME["text_main"],
            hover_color=THEME["border"],
            command=self.select_files
        ).pack(side="left", padx=4)

        ctk.CTkButton(
            btn_box, 
            text="Ajouter Dossier", 
            width=100, 
            height=30, 
            font=ctk.CTkFont(family=FONT_FAMILY, size=11, weight="bold"),
            fg_color=THEME["bg_subcard"],
            text_color=THEME["text_main"],
            hover_color=THEME["border"],
            command=self.select_folder
        ).pack(side="left")

        # Stats Card
        meta_card = ctk.CTkFrame(self.left_panel, fg_color=THEME["bg_subcard"], corner_radius=8)
        meta_card.pack(fill="x", padx=16, pady=4)
        
        self.files_count = ctk.CTkLabel(
            meta_card, 
            text="0 element",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_main"]
        )
        self.files_count.pack(side="left", padx=12, pady=6)
        
        self.total_size_label = ctk.CTkLabel(
            meta_card, 
            text="Taille totale: 0 B", 
            font=ctk.CTkFont(family=FONT_FAMILY, size=12),
            text_color=THEME["text_muted"]
        )
        self.total_size_label.pack(side="right", padx=12, pady=6)

        # Scrollable file list
        self.files_list = ctk.CTkScrollableFrame(
            self.left_panel, 
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"]
        )
        self.files_list.pack(fill="both", expand=True, padx=8, pady=(4, 8))

        # Clear Selection button (no emoji)
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

        # 2. Right Panel (Send configuration)
        self.right_panel = ctk.CTkFrame(
            self,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12
        )

        ctk.CTkLabel(
            self.right_panel, 
            text="Configuration", 
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"]
        ).pack(anchor="w", padx=16, pady=(16, 8))

        # Connection status feedback
        status_box = ctk.CTkFrame(self.right_panel, fg_color=THEME["bg_subcard"], corner_radius=8)
        status_box.pack(fill="x", padx=16, pady=4)
        
        ctk.CTkLabel(
            status_box,
            text="STATUT DU SYSTEME",
            font=ctk.CTkFont(family=FONT_FAMILY, size=9, weight="bold"),
            text_color=THEME["text_dim"]
        ).pack(anchor="w", padx=12, pady=(8, 2))

        self.connection_state = ctk.CTkLabel(
            status_box,
            text="Non connecte",
            font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
            text_color=THEME["danger"],
            wraplength=280,
            justify="left",
        )
        self.connection_state.pack(anchor="w", padx=12, pady=(0, 10))

        # Recipient selection card
        recipient_card = ctk.CTkFrame(self.right_panel, fg_color=THEME["bg_subcard"], corner_radius=8)
        recipient_card.pack(fill="x", padx=16, pady=8)

        ctk.CTkLabel(
            recipient_card,
            text="DESTINATAIRE CIBLE",
            font=ctk.CTkFont(family=FONT_FAMILY, size=9, weight="bold"),
            text_color=THEME["text_dim"]
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
        )
        self.recipient_menu.pack(fill="x", padx=12, pady=(0, 10))

        # Progress dashboard card
        progress_card = ctk.CTkFrame(self.right_panel, fg_color=THEME["bg_subcard"], corner_radius=8)
        progress_card.pack(fill="x", padx=16, pady=4)

        self.progress_label = ctk.CTkLabel(
            progress_card, 
            text="Pret pour l'envoi",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_muted"]
        )
        self.progress_label.pack(anchor="w", padx=12, pady=(8, 4))

        self.progress = ctk.CTkProgressBar(
            progress_card,
            fg_color=THEME["bg_card"],
            progress_color=THEME["primary"]
        )
        self.progress.set(0)
        self.progress.pack(fill="x", padx=12, pady=(0, 12))

        # Refresh recipients & Send buttons (no emojis)
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
            text="Rafraichir",
            height=34,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            fg_color=THEME["bg_subcard"],
            hover_color=THEME["border"],
            command=lambda: self.refresh_recipients(force=True),
        ).pack(fill="x", padx=16, pady=(0, 16))

        # 3. Bottom Panel (History Log - Innovation)
        self.history_panel = ctk.CTkFrame(
            self,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12
        )

        ctk.CTkLabel(
            self.history_panel,
            text="Historique des Transferts",
            font=ctk.CTkFont(family=FONT_FAMILY, size=14, weight="bold"),
            text_color=THEME["text_main"]
        ).pack(anchor="w", padx=16, pady=(10, 4))

        self.history_list = ctk.CTkScrollableFrame(
            self.history_panel,
            fg_color="transparent",
            orientation="vertical",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"]
        )
        self.history_list.pack(fill="both", expand=True, padx=8, pady=(0, 8))

    def _on_configure(self, event):
        if event.widget == self:
            width = event.width
            new_mode = "vertical" if width < 680 else "horizontal"
            if new_mode != self._layout_mode:
                self._layout_mode = new_mode
                self._apply_layout()

    def _apply_layout(self):
        # Responsiveness layout update
        if self._layout_mode == "vertical":
            self.grid_columnconfigure(0, weight=1, minsize=0)
            self.grid_columnconfigure(1, weight=0, minsize=0)
            self.grid_rowconfigure(0, weight=0)
            self.grid_rowconfigure(1, weight=0)
            self.grid_rowconfigure(2, weight=1)
            
            self.left_panel.grid(row=0, column=0, sticky="ew", padx=0, pady=4)
            self.right_panel.grid(row=1, column=0, sticky="ew", padx=0, pady=4)
            self.history_panel.grid(row=2, column=0, sticky="nsew", padx=0, pady=(12, 4))
        else:
            self.grid_columnconfigure(0, weight=2, minsize=0)
            self.grid_columnconfigure(1, weight=1, minsize=0)
            self.grid_rowconfigure(0, weight=4)
            self.grid_rowconfigure(1, weight=2)
            self.grid_rowconfigure(2, weight=0)
            
            self.left_panel.grid(row=0, column=0, sticky="nsew", padx=(0, 8), pady=4)
            self.right_panel.grid(row=0, column=1, sticky="nsew", padx=(8, 0), pady=4)
            self.history_panel.grid(row=1, column=0, columnspan=2, sticky="nsew", padx=2, pady=(12, 4))

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
        if getattr(self, "_is_sending", False):
            self.send_btn.configure(state="disabled", fg_color=THEME["primary"])
            self.clear_btn.configure(state="disabled")
            return
            
        self.clear_btn.configure(state="normal")
        can_transfer, reason = self.controller.can_transfer()
        allowed = can_transfer and len(self.selected_files) > 0
        self.send_btn.configure(
            state="normal" if allowed else "disabled",
            fg_color=THEME["success"] if allowed else THEME["border"]
        )
        if can_transfer:
            self.connection_state.configure(
                text="Actif (Pret pour transfert)", text_color=THEME["success"]
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

        values = ["Everyone (Broadcast)"]
        for p in peers:
            values.append(
                f"{p.get('name', p.get('id', '?'))} • {p.get('ip', '?')}:{p.get('tcp_port', 0)}"
            )

        self._recipient_values = values
        self.recipient_menu.configure(values=values)
        if self.recipient_var.get() not in values:
            self.recipient_var.set(values[0])

        self._update_send_state()

    def select_files(self):
        files = filedialog.askopenfilenames(title="Selectionner des fichiers")
        if files:
            self.selected_files.extend(files)
            self.refresh_selected_files()

    def select_folder(self):
        folder = filedialog.askdirectory(title="Selectionner un dossier")
        if folder:
            self.selected_files.append(folder)
            self.refresh_selected_files()

    def refresh_selected_files(self):
        for w in self.files_list.winfo_children():
            w.destroy()

        count = len(self.selected_files)
        self.files_count.configure(text=f"{count} element{'s' if count > 1 else ''}")

        total = 0
        for p in self.selected_files:
            is_dir, size, _ = self._file_info(p)
            if not is_dir:
                total += size
        self.total_size_label.configure(
            text=f"Taille totale: {self._format_size(total)}"
        )

        if count == 0:
            # Empty state feedback for files (No emojis)
            empty_frame = ctk.CTkFrame(self.files_list, fg_color="transparent")
            empty_frame.pack(fill="both", expand=True, pady=60)
            
            ctk.CTkLabel(
                empty_frame,
                text="Aucun fichier selectionne",
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_muted"]
            ).pack(pady=8)
            
            self._update_send_state()
            return

        for path in self.selected_files:
            is_dir, size, ext = self._file_info(path)
            card = ctk.CTkFrame(
                self.files_list,
                fg_color=THEME["bg_subcard"],
                border_width=1,
                border_color=THEME["border"],
                corner_radius=10
            )
            card.pack(fill="x", padx=4, pady=4)

            name = os.path.basename(path) or path
            kind = "Dossier" if is_dir else f"Fichier {ext.upper() if ext else '<SANS EXT>'}"
            size_text = "--" if is_dir else self._format_size(size)

            card_info = ctk.CTkFrame(card, fg_color="transparent")
            card_info.pack(side="left", fill="both", expand=True, padx=12, pady=10)

            ctk.CTkLabel(
                card_info, 
                text=name, 
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_main"]
            ).pack(anchor="w")
            
            ctk.CTkLabel(
                card_info, 
                text=f"{kind} - {size_text}", 
                font=ctk.CTkFont(family=FONT_FAMILY, size=11),
                text_color=THEME["text_muted"]
            ).pack(anchor="w", pady=(2, 0))

            ctk.CTkButton(
                card,
                text="Retirer",
                width=60,
                height=28,
                font=ctk.CTkFont(family=FONT_FAMILY, size=11, weight="bold"),
                fg_color="transparent",
                text_color=THEME["danger"],
                hover_color=THEME["bg_card"],
                command=lambda p=path: self.remove_file(p),
            ).pack(side="right", padx=10)

        self._update_send_state()

    def remove_file(self, path: str):
        if path in self.selected_files:
            self.selected_files.remove(path)
            self.refresh_selected_files()

    def clear_selection(self):
        self.selected_files.clear()
        self.refresh_selected_files()

    def refresh_history_ui(self):
        for w in self.history_list.winfo_children():
            w.destroy()

        if not self.history:
            ctk.CTkLabel(
                self.history_list,
                text="Aucun transfert effectue pour le moment",
                font=ctk.CTkFont(family=FONT_FAMILY, size=12),
                text_color=THEME["text_dim"]
            ).pack(anchor="w", padx=12, pady=12)
            return

        for item in reversed(self.history):
            card = ctk.CTkFrame(self.history_list, fg_color=THEME["bg_subcard"], corner_radius=6)
            card.pack(fill="x", pady=2)

            # Details grid
            ctk.CTkLabel(
                card,
                text=f"[{item['time']}]",
                font=ctk.CTkFont(family=FONT_FAMILY, size=11),
                text_color=THEME["text_dim"]
            ).pack(side="left", padx=12, pady=4)

            ctk.CTkLabel(
                card,
                text=item["name"],
                font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
                text_color=THEME["text_main"]
            ).pack(side="left", padx=6, pady=4)

            # Status color
            status_color = THEME["success"] if item["status"] == "Reussi" else THEME["danger"]
            ctk.CTkLabel(
                card,
                text=item["status"].upper(),
                font=ctk.CTkFont(family=FONT_FAMILY, size=11, weight="bold"),
                text_color=status_color
            ).pack(side="right", padx=12, pady=4)

            if item["detail"]:
                ctk.CTkLabel(
                    card,
                    text=item["detail"],
                    font=ctk.CTkFont(family=FONT_FAMILY, size=11),
                    text_color=THEME["text_muted"]
                ).pack(side="right", padx=12, pady=4)

    def send_files(self):
        if not self.selected_files:
            CTkMessagebox(
                title="Erreur", message="Aucun fichier selectionne.", icon="warning"
            )
            return

        can_transfer, reason = self.controller.can_transfer()
        if not can_transfer:
            CTkMessagebox(title="Connexion requise", message=reason, icon="warning")
            return

        self._is_sending = True
        self._update_send_state()
        self.progress.configure(progress_color=THEME["primary"])

        self.progress.set(0)
        self.progress_label.configure(text="Progression: 0%")

        import threading
        import queue

        q = queue.Queue()

        def transfer_worker():
            total = len(self.selected_files)
            sent = 0
            failed = []

            for idx, path in enumerate(self.selected_files, start=1):
                ok, msg = self.controller.send_single_file(path, self.recipient_var.get())
                if ok:
                    sent += 1
                else:
                    failed.append(f"{os.path.basename(path)}: {msg}")

                q.put(("file_done", os.path.basename(path), ok, msg))
                q.put(("progress", idx, total))

            q.put(("done", sent, total, failed))

        # Hello la BOP, on lance le transfert en arriere plan pour ne pas bloquer l'UI
        threading.Thread(target=transfer_worker, daemon=True).start()

        self._check_transfer_queue(q)

    def _check_transfer_queue(self, q):
        import queue
        try:
            while True:
                msg = q.get_nowait()
                if msg[0] == "file_done":
                    _, name, ok, detail = msg
                    timestamp = time.strftime("%H:%M:%S")
                    status_str = "Reussi" if ok else "Echec"
                    self.history.append({
                        "name": name,
                        "status": status_str,
                        "time": timestamp,
                        "detail": detail if not ok else "Fichier transfere"
                    })
                    self.refresh_history_ui()
                elif msg[0] == "progress":
                    _, idx, total = msg
                    ratio = idx / total
                    self.progress.set(ratio)
                    self.progress_label.configure(
                        text=f"Progression: {int(ratio * 100)}% ({idx}/{total})"
                    )
                elif msg[0] == "done":
                    _, sent, total, failed = msg
                    self.progress.set(1.0)
                    self.progress.configure(progress_color=THEME["success"])
                    self.progress_label.configure(text=f"Progression: Termine ({sent}/{total})")

                    self._is_sending = False
                    self._update_send_state()

                    if failed:
                        CTkMessagebox(
                            title="Transfert partiel",
                            message=f"{sent}/{total} envoye(s).\n\n"
                            + "\n".join(failed[:8])
                            + ("\n..." if len(failed) > 8 else ""),
                            icon="warning",
                        )
                    else:
                        CTkMessagebox(
                            title="Succes",
                            message=f"{sent}/{total} fichier(s) envoye(s).",
                            icon="check",
                        )
                    return
        except queue.Empty:
            # Re-planifier la verification regulierement
            self.after(50, lambda: self._check_transfer_queue(q))
