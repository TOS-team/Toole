import os
import threading
import time
from tkinter import filedialog
from typing import List

import customtkinter as ctk
from controller.controller import Controller


class TransferScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master)
        self.controller = controller
        self.selected_files: List[str] = []
        self.active_transfers = {}  # Pour suivre les transferts en cours
        self.build_ui()

    def build_ui(self):
        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # ==================== LEFT PANEL ====================
        left = ctk.CTkFrame(self)
        left.grid(row=0, column=0, sticky="nsew", padx=(20, 10), pady=20)

        ctk.CTkLabel(
            left, text="FICHIERS À ENVOYER", font=ctk.CTkFont(size=20, weight="bold")
        ).pack(anchor="w", pady=(10, 15), padx=15)

        btn_frame = ctk.CTkFrame(left, fg_color="transparent")
        btn_frame.pack(fill="x", padx=15, pady=10)

        ctk.CTkButton(
            btn_frame, text="📄 Fichiers", height=48, command=self.select_files
        ).pack(side="left", padx=5, expand=True, fill="x")
        ctk.CTkButton(
            btn_frame, text="📁 Dossier", height=48, command=self.select_folder
        ).pack(side="left", padx=5, expand=True, fill="x")

        self.files_listbox = ctk.CTkScrollableFrame(left)
        self.files_listbox.pack(fill="both", expand=True, padx=15, pady=10)

        ctk.CTkButton(
            left,
            text="🗑️ Vider tout",
            fg_color="#FF5252",
            height=40,
            command=self.clear_selection,
        ).pack(pady=12, padx=15)

        # ==================== RIGHT PANEL ====================
        right = ctk.CTkFrame(self)
        right.grid(row=0, column=1, sticky="nsew", padx=(10, 20), pady=20)

        ctk.CTkLabel(
            right, text="DESTINATAIRE", font=ctk.CTkFont(size=20, weight="bold")
        ).pack(anchor="w", pady=(10, 15), padx=15)

        self.recipient_var = ctk.StringVar(value="Everyone (Broadcast)")

        recipients = [
            "Everyone (Broadcast)",
            "Madeleine_Y (192.168.1.5)",
            "Papa_Z (192.168.1.12)",
            "Admin_BIT (192.168.1.8)",
        ]

        for r in recipients:
            ctk.CTkRadioButton(
                right,
                text=r,
                variable=self.recipient_var,
                value=r,
                font=ctk.CTkFont(size=15),
            ).pack(anchor="w", pady=8, padx=25)

        # Send Button
        self.send_button = ctk.CTkButton(
            right,
            text="🚀 ENVOYER MAINTENANT",
            height=65,
            font=ctk.CTkFont(size=20, weight="bold"),
            command=self.start_transfer,
        )
        self.send_button.pack(side="bottom", fill="x", pady=30, padx=20)

        # ==================== TRANSFER PROGRESS SECTION ====================
        self.progress_frame = ctk.CTkFrame(right)
        self.progress_frame.pack(fill="x", padx=15, pady=(20, 0))

        ctk.CTkLabel(
            self.progress_frame,
            text="TRANSFERTS EN COURS",
            font=ctk.CTkFont(size=16, weight="bold"),
        ).pack(anchor="w", pady=8)

        self.transfers_scroll = ctk.CTkScrollableFrame(self.progress_frame, height=220)
        self.transfers_scroll.pack(fill="x", expand=True)

    # ==================== FILE SELECTION ====================

    def select_files(self):
        files = filedialog.askopenfilenames(title="Sélectionner des fichiers")
        self.add_files(files)

    def select_folder(self):
        folder = filedialog.askdirectory(title="Sélectionner un dossier")
        if folder:
            self.add_files([folder])

    def add_files(self, paths):
        for path in paths:
            if path not in self.selected_files:
                self.selected_files.append(path)
                self.add_file_row(path)

    def add_file_row(self, path: str):
        frame = ctk.CTkFrame(self.files_listbox, fg_color="#2B2B2B", corner_radius=10)
        frame.pack(fill="x", pady=4, padx=8)

        icon = self.get_file_icon(path)
        ctk.CTkLabel(frame, text=icon, font=ctk.CTkFont(size=20)).pack(
            side="left", padx=12
        )

        name = os.path.basename(path)
        if len(name) > 45:
            name = name[:42] + "..."
        ctk.CTkLabel(frame, text=name, anchor="w").pack(
            side="left", padx=8, fill="x", expand=True
        )

        if os.path.isfile(path):
            try:
                size = self.format_size(os.path.getsize(path))
                ctk.CTkLabel(frame, text=size, text_color="gray").pack(
                    side="left", padx=10
                )
            except:
                pass

        ctk.CTkButton(
            frame,
            text="✕",
            width=30,
            height=30,
            fg_color="transparent",
            text_color="#FF5252",
            hover_color="#FF1744",
            command=lambda f=frame, p=path: self.remove_one_item(f, p),
        ).pack(side="right", padx=10)

    def remove_one_item(self, frame, path):
        frame.destroy()
        if path in self.selected_files:
            self.selected_files.remove(path)

    def clear_selection(self):
        for widget in self.files_listbox.winfo_children():
            widget.destroy()
        self.selected_files.clear()

    # ==================== PROGRESS UTILITIES ====================

    def get_file_icon(self, path: str) -> str:
        if os.path.isdir(path):
            return "📁"
        ext = os.path.splitext(path)[1].lower()
        icons = {
            ".png": "🖼️",
            ".jpg": "🖼️",
            ".jpeg": "🖼️",
            ".mp4": "🎥",
            ".zip": "📦",
            ".pdf": "📕",
        }
        return icons.get(ext, "📄")

    def format_size(self, size_bytes: int) -> str:
        for unit in ["B", "KB", "MB", "GB"]:
            if size_bytes < 1024:
                return f"{size_bytes:.1f} {unit}"
            size_bytes /= 1024
        return f"{size_bytes:.1f} TB"

    # ==================== START TRANSFER ====================

    def start_transfer(self):
        if not self.selected_files:
            ctk.CTkMessagebox(
                title="Erreur", message="Aucun fichier sélectionné !", icon="warning"
            )
            return

        recipient = self.recipient_var.get()
        self.send_button.configure(state="disabled", text="ENVOI EN COURS...")

        # Lancer le transfert dans un thread pour ne pas bloquer l'UI
        threading.Thread(
            target=self.simulate_transfer,
            args=(self.selected_files.copy(), recipient),
            daemon=True,
        ).start()

    def simulate_transfer(self, files, recipient):
        for filepath in files:
            filename = os.path.basename(filepath)

            # Créer une ligne de progression
            transfer_id = len(self.active_transfers)
            progress_info = self.create_progress_row(filename, recipient)

            self.active_transfers[transfer_id] = progress_info

            # Simulation de transfert
            total_size = (
                os.path.getsize(filepath) if os.path.isfile(filepath) else 50_000_000
            )
            transferred = 0
            chunk_size = total_size // 30  # ~30 étapes

            for i in range(30):
                transferred += chunk_size
                progress = min((transferred / total_size) * 100, 100)

                # Mise à jour sécurisée de l'UI
                self.after(
                    0,
                    lambda p=progress, info=progress_info, f=filename: (
                        self.update_progress(info, p, f)
                    ),
                )

                time.sleep(0.15)  # Simulation réaliste

            # Transfert terminé
            self.after(0, lambda info=progress_info: self.finish_transfer(info))

        # Fin de tous les transferts
        self.after(0, self.transfer_completed)

    def create_progress_row(self, filename, recipient):
        frame = ctk.CTkFrame(
            self.transfers_scroll, fg_color="#2B2B2B", corner_radius=12
        )
        frame.pack(fill="x", pady=6, padx=8)

        ctk.CTkLabel(frame, text="📤", font=ctk.CTkFont(size=18)).pack(
            side="left", padx=12
        )

        info_frame = ctk.CTkFrame(frame, fg_color="transparent")
        info_frame.pack(side="left", fill="x", expand=True)

        ctk.CTkLabel(
            info_frame, text=filename, font=ctk.CTkFont(size=14, weight="bold")
        ).pack(anchor="w")
        ctk.CTkLabel(
            info_frame,
            text=f"Vers {recipient}",
            text_color="gray",
            font=ctk.CTkFont(size=12),
        ).pack(anchor="w")

        progress_bar = ctk.CTkProgressBar(frame, height=8, corner_radius=4)
        progress_bar.pack(side="left", fill="x", expand=True, padx=15)
        progress_bar.set(0)

        status_label = ctk.CTkLabel(frame, text="0%", width=60, text_color="#4CAF50")
        status_label.pack(side="right", padx=10)

        return {
            "frame": frame,
            "progress_bar": progress_bar,
            "status_label": status_label,
        }

    def update_progress(self, info, progress, filename):
        info["progress_bar"].set(progress / 100)
        info["status_label"].configure(text=f"{int(progress)}%")

    def finish_transfer(self, info):
        info["progress_bar"].set(1)
        info["status_label"].configure(text="✓ Terminé", text_color="#4CAF50")

    def transfer_completed(self):
        self.send_button.configure(state="normal", text="🚀 ENVOYER MAINTENANT")
        ctk.CTkMessagebox(
            title="Succès",
            message="Tous les fichiers ont été envoyés avec succès !",
            icon="check",
        )

    # ==================== UTILITAIRES (inchangés) ====================
    def get_file_icon(self, path: str) -> str:
        if os.path.isdir(path):
            return "📁"
        ext = os.path.splitext(path)[1].lower()
        icons = {
            ".png": "🖼️",
            ".jpg": "🖼️",
            ".jpeg": "🖼️",
            ".mp4": "🎥",
            ".zip": "📦",
            ".pdf": "📕",
        }
        return icons.get(ext, "📄")

    def format_size(self, size_bytes: int) -> str:
        for unit in ["B", "KB", "MB", "GB"]:
            if size_bytes < 1024:
                return f"{size_bytes:.1f} {unit}"
            size_bytes /= 1024
        return f"{size_bytes:.1f} TB"
