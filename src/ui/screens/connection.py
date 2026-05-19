import customtkinter as ctk
from controller.controller import Controller


class ConnectionScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master)
        self.controller = controller

        self.grid_columnconfigure(1, weight=1)
        self.grid_rowconfigure(0, weight=1)

        # Sidebar
        self.sidebar = ctk.CTkFrame(
            self, width=260, corner_radius=0, fg_color="#1F1F1F"
        )
        self.sidebar.grid(row=0, column=0, sticky="nsew")

        ctk.CTkLabel(
            self.sidebar, text="Bit Transfer", font=ctk.CTkFont(size=24, weight="bold")
        ).pack(pady=30)

        ctk.CTkButton(
            self.sidebar,
            text="🔄 Actualiser",
            height=45,
            font=ctk.CTkFont(size=15),
            command=self.refresh_users,
        ).pack(pady=15, padx=25)

        # Main Content
        self.main = ctk.CTkFrame(self, fg_color="transparent")
        self.main.grid(row=0, column=1, sticky="nsew", padx=25, pady=20)

        self.build_configuration()
        self.build_users_section()

    def build_configuration(self):
        config_frame = ctk.CTkFrame(self.main)
        config_frame.pack(fill="x", pady=(0, 30))

        ctk.CTkLabel(
            config_frame, text="CONFIGURATION", font=ctk.CTkFont(size=18, weight="bold")
        ).pack(anchor="w", pady=(0, 15))

        # Username
        ctk.CTkLabel(
            config_frame, text="Nom d'utilisateur", font=ctk.CTkFont(size=14)
        ).pack(anchor="w", padx=5)
        self.username_entry = ctk.CTkEntry(
            config_frame,
            height=42,
            placeholder_text=self.controller.current_user,
            font=ctk.CTkFont(size=15),
        )
        self.username_entry.pack(fill="x", pady=(5, 20), padx=5)

        # Mode
        mode_frame = ctk.CTkFrame(config_frame, fg_color="transparent")
        mode_frame.pack(fill="x", padx=5)
        ctk.CTkLabel(
            mode_frame, text="Mode Automatique", font=ctk.CTkFont(size=15)
        ).pack(side="left")
        self.mode_switch = ctk.CTkSwitch(mode_frame, text="", width=60)
        self.mode_switch.pack(side="right")
        self.mode_switch.select()

    def build_users_section(self):
        ctk.CTkLabel(
            self.main,
            text="UTILISATEURS SUR LE RÉSEAU",
            font=ctk.CTkFont(size=18, weight="bold"),
        ).pack(anchor="w", pady=(0, 15))

        self.users_frame = ctk.CTkScrollableFrame(self.main)
        self.users_frame.pack(fill="both", expand=True)

        self.refresh_users()

    def refresh_users(self):
        for widget in self.users_frame.winfo_children():
            widget.destroy()

        users = [
            ("Madeleine_Y", "192.168.1.5", "En attente", "#FF9800"),
            ("Papa_Z", "192.168.1.12", "Connecté", "#4CAF50"),
            ("Admin_BIT", "192.168.1.8", "Disponible", "#2196F3"),
        ]

        for name, ip, status, color in users:
            self.create_user_card(name, ip, status, color)

    def create_user_card(self, name, ip, status, color):
        card = ctk.CTkFrame(self.users_frame, height=85, corner_radius=16)
        card.pack(fill="x", pady=8, padx=8)
        card.grid_columnconfigure(1, weight=1)

        ctk.CTkLabel(card, text="👤", font=ctk.CTkFont(size=32)).grid(
            row=0, column=0, padx=20, pady=10, rowspan=2
        )

        ctk.CTkLabel(card, text=name, font=ctk.CTkFont(size=17, weight="bold")).grid(
            row=0, column=1, sticky="w"
        )
        ctk.CTkLabel(card, text=ip, text_color="gray").grid(row=1, column=1, sticky="w")

        ctk.CTkLabel(
            card, text=status, text_color=color, font=ctk.CTkFont(size=14)
        ).grid(row=0, column=2, padx=20)

        ctk.CTkButton(
            card,
            text="Se Connecter",
            width=130,
            height=38,
            command=lambda n=name, i=ip: self.controller.accept_connection(n, i),
        ).grid(row=0, column=3, padx=15, rowspan=2)
