import time
from tkinter import messagebox

import customtkinter as ctk
from controller.controller import Controller
from screens.connection import ConnectionScreen
from screens.reception import ReceptionScreen
from screens.transfer import TransferScreen
from theme import FONT_FAMILY, THEME


class App(ctk.CTk):
    def __init__(self):
        super().__init__()

        self.title("Toole P2P File Transfer")
        self.geometry("1040x680")
        self.minsize(520, 440)

        ctk.set_appearance_mode("dark")
        self.configure(fg_color=THEME["bg_dark"])

        self.controller = Controller()
        self._runtime_tick_ms = 250
        self._ui_refresh_ms = 1000
        self._last_ui_refresh = 0.0

        if not self.controller.start_backend():
            messagebox.showerror(
                "Backend Error",
                f"Impossible de demarrer le backend:\n{self.controller.get_last_error()}",
            )

        self._build_ui()
        self.protocol("WM_DELETE_WINDOW", self.on_close)
        self.after(self._runtime_tick_ms, self._runtime_loop)

    def _build_ui(self):
        root = ctk.CTkFrame(self, fg_color="transparent")
        root.pack(fill="both", expand=True, padx=12, pady=12)

        top_bar = ctk.CTkFrame(root, fg_color="transparent")
        top_bar.pack(fill="x", pady=(0, 10))

        title_box = ctk.CTkFrame(top_bar, fg_color="transparent")
        title_box.pack(side="left", fill="x", expand=True)

        ctk.CTkLabel(
            title_box,
            text="TOOLÉ",
            font=ctk.CTkFont(family=FONT_FAMILY, size=24, weight="bold"),
            text_color=THEME["primary"],
        ).pack(anchor="w")

        self.local_info = ctk.CTkLabel(
            title_box,
            text=f"{self.controller.current_user} • {self.controller.local_ip}:{self.controller.tcp_port}",
            font=ctk.CTkFont(family=FONT_FAMILY, size=11),
            text_color=THEME["text_muted"],
            anchor="w",
            justify="left",
        )
        self.local_info.pack(anchor="w")

        self.status_badge = ctk.CTkFrame(
            top_bar,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=20,
        )
        self.status_badge.pack(side="right", padx=4, pady=4)

        self.top_status = ctk.CTkLabel(
            self.status_badge,
            text="Initialisation...",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["text_muted"],
            padx=12,
            pady=4,
        )
        self.top_status.pack()

        # Hello la BOP, on utilise un TabView natif: moins de logique maison,
        # donc un comportement plus stable quand la fenetre devient petite.
        self.tabs = ctk.CTkTabview(
            root,
            fg_color=THEME["bg_card"],
            segmented_button_fg_color=THEME["bg_subcard"],
            segmented_button_selected_color=THEME["primary"],
            segmented_button_selected_hover_color=THEME["primary_hover"],
            segmented_button_unselected_color=THEME["bg_subcard"],
            segmented_button_unselected_hover_color=THEME["border"],
            text_color=THEME["text_main"],
        )
        self.tabs.pack(fill="both", expand=True)

        self.tab_connexion = self.tabs.add("Connexion")
        self.tab_transfert = self.tabs.add("Transfert")
        self.tab_reception = self.tabs.add("Réception")

        self.screens = {
            "Connexion": ConnectionScreen(self.tab_connexion, self.controller),
            "Transfert": TransferScreen(self.tab_transfert, self.controller),
            "Réception": ReceptionScreen(self.tab_reception, self.controller),
        }

        for screen in self.screens.values():
            screen.pack(fill="both", expand=True, padx=4, pady=4)

    def _refresh_runtime_ui(self):
        self.screens["Connexion"].refresh_users(force=False)
        self.screens["Transfert"].refresh_recipients(force=False)
        self.screens["Réception"].refresh_received()

        snap, _ = self.controller.get_runtime_view()
        if not snap:
            self.top_status.configure(text="Backend...", text_color=THEME["warning"])
            return

        state_val = snap.get("state", -1)
        state_names = {
            0: "DISCOVERING",
            1: "CONNECTING",
            2: "CLIENT",
            3: "MASTER",
            4: "ELECTION",
        }
        state_text = state_names.get(state_val, "UNKNOWN")
        role_text = "MASTER" if snap.get("role") == 1 else "CLIENT"

        color = THEME["primary"]
        if state_val == 2:
            color = THEME["success"]
        elif state_val == 3:
            color = "#A855F7"
        elif state_val == 0:
            color = THEME["warning"]

        self.top_status.configure(text=f"{state_text} | {role_text}", text_color=color)
        self.local_info.configure(
            text=f"{self.controller.current_user} • {self.controller.local_ip}:{self.controller.tcp_port}"
        )

    def _runtime_loop(self):
        self.controller.tick_backend()

        now = time.monotonic()
        if now - self._last_ui_refresh >= self._ui_refresh_ms / 1000:
            self._last_ui_refresh = now
            self._refresh_runtime_ui()

        self.after(self._runtime_tick_ms, self._runtime_loop)

    def on_close(self):
        self.controller.stop_backend()
        self.destroy()


if __name__ == "__main__":
    app = App()
    app.mainloop()
