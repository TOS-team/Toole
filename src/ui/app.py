from tkinter import messagebox
import customtkinter as ctk
from controller.controller import Controller
from screens.connection import ConnectionScreen
from screens.transfer import TransferScreen

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

class App(ctk.CTk):
    def __init__(self):
        super().__init__()
        
        # Configure window basic settings (Allows maximum possible shrinking now)
        self.title("Toole P2P File Transfer")
        self.geometry("980x640")
        self.minsize(550, 480)
        
        # Apply dark mode and theme colors
        ctk.set_appearance_mode("dark")
        self.configure(fg_color=THEME["bg_dark"])

        self.controller = Controller()
        self._ui_refresh_ms = 1800

        if not self.controller.start_backend():
            messagebox.showerror(
                "Backend Error",
                f"Impossible de demarrer le backend:\n{self.controller.get_last_error()}",
            )

        self._active_button = None
        self._build_ui()
        self.protocol("WM_DELETE_WINDOW", self.on_close)
        
        # Bind to configure event for responsive window adjustments
        self.bind("<Configure>", self._on_window_configure)
        self.after(self._ui_refresh_ms, self._runtime_loop)

    def _build_ui(self):
        # Main layout frame
        self.main_layout = ctk.CTkFrame(self, fg_color="transparent")
        self.main_layout.pack(fill="both", expand=True)

        # 1. Sidebar Panel (Navigation)
        self.sidebar = ctk.CTkFrame(
            self.main_layout, 
            width=220, 
            fg_color=THEME["bg_card"], 
            border_width=1, 
            border_color=THEME["border"],
            corner_radius=0
        )
        self.sidebar.pack(side="left", fill="y")
        self.sidebar.pack_propagate(False)

        # Logo / Title Header (No emojis)
        self.logo_frame = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        self.logo_frame.pack(fill="x", padx=16, pady=24)
        
        self.lbl_logo = ctk.CTkLabel(
            self.logo_frame,
            text="TOOLÉ",
            font=ctk.CTkFont(family=FONT_FAMILY, size=24, weight="bold"),
            text_color=THEME["primary"]
        )
        self.lbl_logo.pack(anchor="w")

        self.lbl_subtitle = ctk.CTkLabel(
            self.logo_frame,
            text="P2P LAN Transfer",
            font=ctk.CTkFont(family=FONT_FAMILY, size=11),
            text_color=THEME["text_dim"]
        )
        self.lbl_subtitle.pack(anchor="w", pady=(2, 0))

        # Navigation Buttons Container
        self.nav_frame = ctk.CTkFrame(self.sidebar, fg_color="transparent")
        self.nav_frame.pack(fill="both", expand=True, padx=12, pady=10)

        self.btn_conn = self._create_nav_button("Connexion", "Connexion")
        self.btn_tran = self._create_nav_button("Transfert", "Transfert")

        # Sidebar Footer (Local Info - No emojis)
        self.footer = ctk.CTkFrame(self.sidebar, fg_color=THEME["bg_subcard"], corner_radius=8, height=80)
        self.footer.pack(side="bottom", fill="x", padx=12, pady=16)
        self.footer.pack_propagate(False)

        self.lbl_user = ctk.CTkLabel(
            self.footer,
            text=f"User: {self.controller.current_user}",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_main"]
        )
        self.lbl_user.pack(anchor="w", padx=10, pady=(10, 2))

        self.lbl_ip = ctk.CTkLabel(
            self.footer,
            text=f"IP: {self.controller.local_ip}",
            font=ctk.CTkFont(family=FONT_FAMILY, size=11),
            text_color=THEME["text_muted"]
        )
        self.lbl_ip.pack(anchor="w", padx=10)

        # 2. Main Content Panel
        self.content_area = ctk.CTkFrame(self.main_layout, fg_color="transparent")
        self.content_area.pack(side="right", fill="both", expand=True, padx=16, pady=16)

        # Top Bar (Header & Status)
        self.top_bar = ctk.CTkFrame(self.content_area, fg_color="transparent")
        self.top_bar.pack(fill="x", pady=(0, 16))

        self.lbl_screen_title = ctk.CTkLabel(
            self.top_bar,
            text="Connexion",
            font=ctk.CTkFont(family=FONT_FAMILY, size=24, weight="bold"),
            text_color=THEME["text_main"]
        )
        self.lbl_screen_title.pack(side="left")

        # Status badge container
        self.status_badge = ctk.CTkFrame(
            self.top_bar,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=20
        )
        self.status_badge.pack(side="right", padx=4, pady=4)

        self.top_status = ctk.CTkLabel(
            self.status_badge,
            text="Initialisation...",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_muted"],
            padx=12,
            pady=4
        )
        self.top_status.pack()

        # Screens Container
        self.screens_container = ctk.CTkFrame(self.content_area, fg_color="transparent")
        self.screens_container.pack(fill="both", expand=True)

        self.screens = {
            "Connexion": ConnectionScreen(self.screens_container, self.controller),
            "Transfert": TransferScreen(self.screens_container, self.controller)
        }

        # Show initial screen
        self.show_screen("Connexion")

    def _create_nav_button(self, text, screen_name):
        btn = ctk.CTkButton(
            self.nav_frame,
            text=text,
            height=40,
            corner_radius=8,
            anchor="w",
            font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
            fg_color="transparent",
            text_color=THEME["text_muted"],
            hover_color=THEME["bg_subcard"],
            command=lambda: self.show_screen(screen_name)
        )
        btn.pack(fill="x", pady=4)
        return btn

    def show_screen(self, name):
        # Hide all screens
        for screen in self.screens.values():
            screen.pack_forget()
            
        # Show target screen
        self.screens[name].pack(fill="both", expand=True)
        self.lbl_screen_title.configure(text=name)

        # Highlight navigation buttons
        self.btn_conn.configure(
            fg_color=THEME["primary"] if name == "Connexion" else "transparent",
            text_color=THEME["text_main"] if name == "Connexion" else THEME["text_muted"]
        )
        self.btn_tran.configure(
            fg_color=THEME["primary"] if name == "Transfert" else "transparent",
            text_color=THEME["text_main"] if name == "Transfert" else THEME["text_muted"]
        )

    def _on_window_configure(self, event):
        # Handle only the main window configuration
        if event.widget == self:
            width = event.width
            # Hello la BOP, on adapte dynamiquement la sidebar si l'ecran devient petit
            if width < 720:
                self.sidebar.configure(width=140)
                self.lbl_logo.configure(font=ctk.CTkFont(family=FONT_FAMILY, size=18, weight="bold"))
                self.lbl_subtitle.pack_forget()
                self.footer.pack_forget()
            else:
                self.sidebar.configure(width=220)
                self.lbl_logo.configure(font=ctk.CTkFont(family=FONT_FAMILY, size=24, weight="bold"))
                self.lbl_subtitle.pack(anchor="w", pady=(2, 0))
                # Put footer back if not showing
                if not self.footer.winfo_ismapped():
                    self.footer.pack(side="bottom", fill="x", padx=12, pady=16)

    def _runtime_loop(self):
        self.controller.tick_backend()
        self.connection_screen = self.screens["Connexion"]
        self.transfer_screen = self.screens["Transfert"]
        
        self.connection_screen.refresh_users(force=False)
        self.transfer_screen.refresh_recipients(force=False)

        snap, _ = self.controller.get_runtime_view()
        if snap:
            state_val = snap.get("state", -1)
            state_names = {0: "DISCOVERING", 1: "CONNECTING", 2: "CLIENT", 3: "MASTER", 4: "ELECTION"}
            state_text = state_names.get(state_val, "UNKNOWN")
            role_text = "MASTER" if snap.get("role") == 1 else "CLIENT"
            
            # Choose badge color based on state
            color = THEME["primary"]
            if state_val == 2:  # CLIENT
                color = THEME["success"]
            elif state_val == 3:  # MASTER
                color = "#A855F7"  # Elegant Purple for master
            elif state_val == 0:  # DISCOVERING
                color = THEME["warning"]
                
            self.top_status.configure(
                text=f"{state_text} | {role_text}",
                text_color=color
            )
            
            # Refresh local footer user name if modified
            self.lbl_user.configure(text=f"User: {self.controller.current_user}")

        self.after(self._ui_refresh_ms, self._runtime_loop)

    def on_close(self):
        self.controller.stop_backend()
        self.destroy()

if __name__ == "__main__":
    app = App()
    app.mainloop()
