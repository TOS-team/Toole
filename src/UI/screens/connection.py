import customtkinter as ctk
from controller.controller import Controller

class ConnectionScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master)
        self.controller = controller

        # Sidebar
        sidebar = ctk.CTkFrame(self, width=200, corner_radius=0)
        sidebar.pack(side="left", fill="y")

        ctk.CTkLabel(sidebar, text="Connexions", font=ctk.CTkFont(size=20, weight="bold")).pack(pady=20)

        ctk.CTkButton(sidebar, text="🔄 Actualiser", command=self.refresh_users).pack(pady=10, padx=20)

        # Main content
        main = ctk.CTkFrame(self)
        main.pack(side="right", fill="both", expand=True, padx=20, pady=20)

        # Configuration
        config_frame = ctk.CTkFrame(main)
        config_frame.pack(fill="x", pady=(0, 20))

        ctk.CTkLabel(config_frame, text="CONFIGURATION", font=ctk.CTkFont(size=16, weight="bold")).pack(anchor="w", pady=5)

        ctk.CTkLabel(config_frame, text="Nom d'utilisateur :").pack(anchor="w", padx=10)
        self.username_entry = ctk.CTkEntry(config_frame, placeholder_text=self.controller.current_user)
        self.username_entry.pack(fill="x", padx=10, pady=5)

        # Mode
        mode_frame = ctk.CTkFrame(config_frame)
        mode_frame.pack(fill="x", padx=10, pady=10)
        ctk.CTkLabel(mode_frame, text="Mode :").pack(side="left", padx=5)
        self.mode_switch = ctk.CTkSwitch(mode_frame, text="AUTOMATIQUE", onvalue="on", offvalue="off")
        self.mode_switch.pack(side="right")

        # Available users
        ctk.CTkLabel(main, text="UTILISATEURS DISPONIBLES (Réseau local)", 
                    font=ctk.CTkFont(size=16, weight="bold")).pack(anchor="w", pady=(20,10))

        self.users_frame = ctk.CTkScrollableFrame(main, height=300)
        self.users_frame.pack(fill="both", expand=True)

        self.refresh_users()

    def refresh_users(self):
        for widget in self.users_frame.winfo_children():
            widget.destroy()

        users = [
            ("Madeleine_Y", "192.168.1.5", "Pending acceptance..."),
            ("Papa_Z", "192.168.1.??", "Connected"),
            ("Admin_BIT", "192.168.1.??", "Available"),
        ]

        for name, ip, status in users:
            user_frame = ctk.CTkFrame(self.users_frame)
            user_frame.pack(fill="x", pady=5, padx=5)

            ctk.CTkLabel(user_frame, text=name, font=ctk.CTkFont(size=15)).pack(side="left", padx=10)
            ctk.CTkLabel(user_frame, text=ip, text_color="gray").pack(side="left", padx=10)
            
            status_label = ctk.CTkLabel(user_frame, text=status, text_color="orange" if "Pending" in status else "green")
            status_label.pack(side="right", padx=10)