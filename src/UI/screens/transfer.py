import customtkinter as ctk
from tkinter import filedialog
import os
from typing import List
from controller.controller import Controller


class TransferScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master)
        self.controller = controller
        self.selected_files: List[str] = []   # Liste des chemins sélectionnés

        self.build_ui()

    def build_ui(self):
        # Panneau gauche - Sélection
        left = ctk.CTkFrame(self, width=380)
        left.pack(side="left", fill="y", padx=(20, 10), pady=20)

        ctk.CTkLabel(left, text="SÉLECTION DE FICHIERS", 
                    font=ctk.CTkFont(size=18, weight="bold")).pack(anchor="w", pady=(0, 15))

        # Boutons sélection
        btn_frame = ctk.CTkFrame(left, fg_color="transparent")
        btn_frame.pack(fill="x", pady=10, padx=10)

        ctk.CTkButton(btn_frame, text="📄 Sélectionner un fichier", 
                     command=self.select_files).pack(side="left", padx=5, fill="x", expand=True)

        ctk.CTkButton(btn_frame, text="📁 Sélectionner un dossier", 
                     command=self.select_folder).pack(side="left", padx=5, fill="x", expand=True)

        # Liste des fichiers sélectionnés
        ctk.CTkLabel(left, text="Fichiers / Dossiers sélectionnés :", 
                    font=ctk.CTkFont(size=14, weight="bold")).pack(anchor="w", pady=(20, 5), padx=10)

        self.files_listbox = ctk.CTkScrollableFrame(left, height=320)
        self.files_listbox.pack(fill="both", expand=True, padx=10, pady=5)

        ctk.CTkButton(left, text="🗑️ Tout supprimer", 
                     fg_color="#FF5252", hover_color="#FF1744",
                     command=self.clear_selection).pack(pady=10)

        # Panneau droit - Destinataire
        right = ctk.CTkFrame(self)
        right.pack(side="right", fill="both", expand=True, padx=(10, 20), pady=20)

        ctk.CTkLabel(right, text="CHOISIR UN DESTINATAIRE", 
                    font=ctk.CTkFont(size=16, weight="bold")).pack(anchor="w", pady=(0, 10))

        self.recipient_var = ctk.StringVar(value="Everyone (Broadcast)")

        recipients = ["Everyone (Broadcast)", "Madeleine_Y 192.168.1.5", "Papa_Z 192.168.1.10"]
        for r in recipients:
            ctk.CTkRadioButton(right, text=r, variable=self.recipient_var, 
                             value=r).pack(anchor="w", pady=6, padx=15)

        # Bouton Envoyer
        self.send_button = ctk.CTkButton(right, text="🚀 ENVOYER", height=55,
                                        font=ctk.CTkFont(size=18, weight="bold"),
                                        command=self.send_files)
        self.send_button.pack(side="bottom", fill="x", pady=30, padx=20)

    # ==================== NOUVELLES MÉTHODES ====================

    def select_files(self):
        files = filedialog.askopenfilenames(title="Sélectionner des fichiers")
        if files:
            self.selected_files.extend(files)
            self.refresh_selected_files()

    def select_folder(self):
        folder = filedialog.askdirectory(title="Sélectionner un dossier")
        if folder:
            self.selected_files.append(folder)
            self.refresh_selected_files()

    def refresh_selected_files(self):
        """Affiche les fichiers avec icônes adaptées"""
        for widget in self.files_listbox.winfo_children():
            widget.destroy()

        for path in self.selected_files:
            frame = ctk.CTkFrame(self.files_listbox, fg_color="#2B2B2B", corner_radius=6)
            frame.pack(fill="x", pady=3, padx=8)

            # Icône selon le type
            icon = self.get_file_icon(path)
            ctk.CTkLabel(frame, text=icon, font=ctk.CTkFont(size=18)).pack(side="left", padx=(10, 5))

            # Nom
            name = os.path.basename(path)
            if len(name) > 45:
                name = name[:42] + "..."
            ctk.CTkLabel(frame, text=name, anchor="w").pack(side="left", padx=5, fill="x", expand=True)

            # Taille (si c'est un fichier)
            try:
                if os.path.isfile(path):
                    size_str = self.format_size(os.path.getsize(path))
                    ctk.CTkLabel(frame, text=size_str, text_color="gray").pack(side="left", padx=10)
            except:
                pass

            # Bouton supprimer
            ctk.CTkButton(frame, text="✕", width=30, height=28,
                         fg_color="transparent", text_color="#FF5252",
                         hover_color="#FF1744",
                         command=lambda p=path: self.remove_file(p)).pack(side="right", padx=8)

    def get_file_icon(self, path: str) -> str:
        """Retourne l'icône selon le type de fichier/dossier"""
        if os.path.isdir(path):
            return "📁"
        
        ext = os.path.splitext(path)[1].lower()
        
        if ext in ['.png', '.jpg', '.jpeg', '.gif', '.bmp', '.webp']:
            return "🖼️"
        elif ext in ['.mp4', '.mkv', '.avi', '.mov']:
            return "🎥"
        elif ext in ['.mp3', '.wav']:
            return "🎵"
        elif ext in ['.py', '.c', '.cpp', '.java', '.js']:
            return "💻"
        elif ext in ['.pdf']:
            return "📕"
        elif ext in ['.zip', '.rar', '.7z', '.iso']:
            return "📦"
        else:
            return "📄"

    def format_size(self, size_bytes: int) -> str:
        for unit in ['B', 'KB', 'MB', 'GB']:
            if size_bytes < 1024:
                return f"{size_bytes:.1f} {unit}"
            size_bytes /= 1024
        return f"{size_bytes:.1f} TB"

    def remove_file(self, path: str):
        if path in self.selected_files:
            self.selected_files.remove(path)
            self.refresh_selected_files()

    def clear_selection(self):
        self.selected_files.clear()
        self.refresh_selected_files()

    def send_files(self):
        if not self.selected_files:
            ctk.CTkMessagebox(title="Erreur", message="Aucun fichier sélectionné !", icon="warning")
            return

        recipient = self.recipient_var.get()
        self.controller.send_files(self.selected_files, recipient)