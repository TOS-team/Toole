import customtkinter as ctk
from controller.controller import Controller
from theme import FONT_FAMILY, THEME


class ActivityScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master, fg_color="transparent")
        self.controller = controller

        self.body = ctk.CTkFrame(self, fg_color="transparent")
        self.body.pack(fill="both", expand=True)

        card = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )
        card.pack(fill="both", expand=True)

        ctk.CTkLabel(
            card,
            text="Evenements",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(anchor="w", padx=16, pady=(16, 8))

        self.events_list = ctk.CTkScrollableFrame(
            card,
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"],
        )
        self.events_list.pack(fill="both", expand=True, padx=8, pady=(0, 8))

        self.refresh_events()

    def refresh_events(self):
        for widget in self.events_list.winfo_children():
            widget.destroy()

        events = self.controller.get_events()
        if not events:
            ctk.CTkLabel(
                self.events_list,
                text="Aucun evenement pour le moment",
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_muted"],
            ).pack(anchor="w", padx=12, pady=12)
            return

        for event in events:
            kind = event.get("kind", "info")
            color = THEME["text_muted"]
            if kind in {"send_done", "received", "peer_joined"}:
                color = THEME["success"]
            elif kind in {"send_failed", "peer_left"}:
                color = THEME["danger"]
            elif kind == "send_partial":
                color = THEME["warning"]

            card = ctk.CTkFrame(
                self.events_list,
                fg_color=THEME["bg_subcard"],
                border_width=1,
                border_color=THEME["border"],
                corner_radius=10,
            )
            card.pack(fill="x", padx=4, pady=4)

            ctk.CTkLabel(
                card,
                text=f"[{event.get('time', '--:--:--')}] {event.get('message', '')}",
                font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
                text_color=color,
                anchor="w",
                justify="left",
                wraplength=720,
            ).pack(anchor="w", fill="x", padx=12, pady=10)
