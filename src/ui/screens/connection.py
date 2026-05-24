import customtkinter as ctk
from controller.controller import Controller
from CTkMessagebox import CTkMessagebox
from theme import FONT_FAMILY, THEME


def _state_name(value: int) -> str:
    return {
        0: "RECHERCHE",
        1: "CONNEXION",
        2: "CLIENT",
        3: "MASTER",
        4: "ELECTION",
    }.get(value, str(value))



class ConnectionScreen(ctk.CTkFrame):
    def __init__(self, master, controller: Controller):
        super().__init__(master, fg_color="transparent")
        self.controller = controller
        self._last_signature = None
        self._layout_mode = "horizontal"

        # Hello la BOP, l'ecran est scrollable pour ne jamais perdre
        # les cartes quand la fenetre devient petite.
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
        self.refresh_users(force=True)

    def _build_ui(self):
        # 1. Left Panel (Dashboard & Profile)
        self.left_panel = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )

        ctk.CTkLabel(
            self.left_panel,
            text="Mon Profil",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(anchor="w", padx=16, pady=(16, 8))

        # Username Card
        user_card = ctk.CTkFrame(
            self.left_panel, fg_color=THEME["bg_subcard"], corner_radius=8
        )
        user_card.pack(fill="x", padx=16, pady=4)

        ctk.CTkLabel(
            user_card,
            text="NOM D'UTILISATEUR",
            font=ctk.CTkFont(family=FONT_FAMILY, size=9, weight="bold"),
            text_color=THEME["text_dim"],
        ).pack(anchor="w", padx=12, pady=(8, 2))

        self.username_entry = ctk.CTkEntry(
            user_card,
            height=32,
            fg_color=THEME["bg_card"],
            border_color=THEME["border"],
            font=ctk.CTkFont(family=FONT_FAMILY, size=13),
            text_color=THEME["text_main"],
        )
        self.username_entry.pack(fill="x", padx=12, pady=(0, 8))
        self.username_entry.insert(0, self.controller.current_user)
        self.username_entry.bind("<FocusOut>", self._update_username)
        self.username_entry.bind("<Return>", self._update_username)

        # Dashboard status items
        ctk.CTkLabel(
            self.left_panel,
            text="Statut Reseau",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(anchor="w", padx=16, pady=(16, 8))

        self.status_cards_frame = ctk.CTkFrame(self.left_panel, fg_color="transparent")
        self.status_cards_frame.pack(fill="both", expand=True, padx=16)

        # Define status labels
        self.lbl_state = self._add_dashboard_item("ETAT ACTUEL", "?", THEME["warning"])
        self.lbl_role = self._add_dashboard_item("ROLE NOEUD", "?", THEME["primary"])
        self.lbl_cluster = self._add_dashboard_item(
            "CLUSTER ID", "-", THEME["text_muted"]
        )
        self.lbl_master = self._add_dashboard_item(
            "IP MASTER", "-", THEME["text_muted"]
        )

        # Refresh Profile button
        ctk.CTkButton(
            self.left_panel,
            text="Actualiser",
            height=36,
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            fg_color=THEME["primary"],
            hover_color=THEME["primary_hover"],
            command=lambda: self.refresh_users(force=True),
        ).pack(fill="x", padx=16, pady=16)

        # 2. Right Panel (Peer list)
        self.right_panel = ctk.CTkFrame(
            self.body,
            fg_color=THEME["bg_card"],
            border_width=1,
            border_color=THEME["border"],
            corner_radius=12,
        )

        head = ctk.CTkFrame(self.right_panel, fg_color="transparent")
        head.pack(fill="x", padx=16, pady=(16, 8))

        ctk.CTkLabel(
            head,
            text="Appareils Detectes",
            font=ctk.CTkFont(family=FONT_FAMILY, size=16, weight="bold"),
            text_color=THEME["text_main"],
        ).pack(side="left")

        # Sleek badge count
        self.count_badge = ctk.CTkFrame(
            head, fg_color=THEME["bg_subcard"], corner_radius=12
        )
        self.count_badge.pack(side="right")
        self.count_label = ctk.CTkLabel(
            self.count_badge,
            text="0",
            font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
            text_color=THEME["primary"],
            padx=10,
            pady=2,
        )
        self.count_label.pack()

        # Search Bar Container (Innovation)
        search_frame = ctk.CTkFrame(self.right_panel, fg_color="transparent")
        search_frame.pack(fill="x", padx=16, pady=(0, 8))

        self.search_var = ctk.StringVar()
        self.search_var.trace_add("write", lambda *args: self.refresh_users(force=True))

        self.search_entry = ctk.CTkEntry(
            search_frame,
            textvariable=self.search_var,
            placeholder_text="Rechercher un appareil par nom ou IP...",
            height=32,
            fg_color=THEME["bg_subcard"],
            border_color=THEME["border"],
            font=ctk.CTkFont(family=FONT_FAMILY, size=12),
            text_color=THEME["text_main"],
        )
        self.search_entry.pack(fill="x")

        # Scrollable peer list
        self.users_frame = ctk.CTkScrollableFrame(
            self.right_panel,
            fg_color="transparent",
            scrollbar_button_color=THEME["border"],
            scrollbar_button_hover_color=THEME["bg_subcard"],
        )
        self.users_frame.pack(fill="both", expand=True, padx=8, pady=(0, 8))

    def _on_configure(self, event):
        if event.widget != self:
            return
        width = self.winfo_width()
        if width <= 1:
            return
        new_mode = "vertical" if width < 760 else "horizontal"
        if new_mode != self._layout_mode:
            self._layout_mode = new_mode
            self._apply_layout()
            self.refresh_users(force=True)

    def _apply_layout(self):
        # On oublie l'ancienne grille avant de changer de mode,
        # sinon Tk garde des contraintes qui cassent le responsive.
        self.left_panel.grid_forget()
        self.right_panel.grid_forget()
        for i in range(2):
            self.body.grid_columnconfigure(i, weight=0, minsize=0)
            self.body.grid_rowconfigure(i, weight=0, minsize=0)

        if self._layout_mode == "vertical":
            self.body.grid_columnconfigure(0, weight=1, minsize=0)
            self.body.grid_rowconfigure(0, weight=0, minsize=0)
            self.body.grid_rowconfigure(1, weight=1, minsize=180)

            self.left_panel.grid(row=0, column=0, sticky="ew", padx=0, pady=(0, 8))
            self.right_panel.grid(row=1, column=0, sticky="nsew", padx=0, pady=(8, 0))
        else:
            self.body.grid_columnconfigure(0, weight=1, minsize=230)
            self.body.grid_columnconfigure(1, weight=2, minsize=280)
            self.body.grid_rowconfigure(0, weight=1, minsize=0)

            self.left_panel.grid(row=0, column=0, sticky="nsew", padx=(0, 8), pady=4)
            self.right_panel.grid(row=0, column=1, sticky="nsew", padx=(8, 0), pady=4)

    def _add_dashboard_item(self, title, default_val, text_color):
        card = ctk.CTkFrame(
            self.status_cards_frame, fg_color=THEME["bg_subcard"], corner_radius=8
        )
        card.pack(fill="x", pady=4)

        ctk.CTkLabel(
            card,
            text=title,
            font=ctk.CTkFont(family=FONT_FAMILY, size=9, weight="bold"),
            text_color=THEME["text_dim"],
        ).pack(anchor="w", padx=12, pady=(6, 2))

        val_label = ctk.CTkLabel(
            card,
            text=default_val,
            font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
            text_color=text_color,
        )
        val_label.pack(anchor="w", padx=12, pady=(0, 6))
        return val_label

    def _update_username(self, event=None):
        new_name = self.username_entry.get().strip()
        if not new_name:
            self.username_entry.delete(0, "end")
            self.username_entry.insert(0, self.controller.current_user)
            return

        ok, msg = self.controller.update_username(new_name)
        if not ok:
            CTkMessagebox(title="Profil", message=msg, icon="warning")
            self.username_entry.delete(0, "end")
            self.username_entry.insert(0, self.controller.current_user)
        self.refresh_users(force=True)

    def _signature(self, snapshot, peers):
        sig_peers = tuple(
            (p.get("id"), p.get("ip"), p.get("role"), p.get("tcp_port")) for p in peers
        )
        sig_snap = (
            snapshot.get("state") if snapshot else None,
            snapshot.get("role") if snapshot else None,
            snapshot.get("cluster_id") if snapshot else None,
            snapshot.get("master_ip") if snapshot else None,
            snapshot.get("master_port") if snapshot else None,
        )
        return sig_snap, sig_peers

    def _connect_to_peer(self, peer):
        ok, msg = self.controller.connect_to_peer(peer)
        CTkMessagebox(
            title="Connexion" if ok else "Erreur",
            message=msg,
            icon="check" if ok else "warning",
        )
        self.refresh_users(force=True)

    def refresh_users(self, force: bool = False):
        snapshot, peers = self.controller.get_runtime_view()
        sig = self._signature(snapshot, peers)
        if not force and sig == self._last_signature:
            return
        self._last_signature = sig

        state_val = snapshot.get("state", -1) if snapshot else -1
        state_txt = _state_name(state_val)
        role_txt = "MASTER" if snapshot and snapshot.get("role") == 1 else "CLIENT"

        # Update dashboard items
        self.lbl_state.configure(text=state_txt)
        if state_val == 2:  # CLIENT
            self.lbl_state.configure(text_color=THEME["success"])
        elif state_val == 3:  # MASTER
            self.lbl_state.configure(text_color="#A855F7")
        else:
            self.lbl_state.configure(text_color=THEME["warning"])

        self.lbl_role.configure(text=role_txt)
        self.lbl_cluster.configure(
            text=snapshot.get("cluster_id", "-") if snapshot else "-"
        )
        self.lbl_master.configure(
            text=f"{snapshot.get('master_ip', '-')}:{snapshot.get('master_port', '-')}"
            if snapshot and snapshot.get("master_port", 0) > 0
            else "-"
        )

        for w in self.users_frame.winfo_children():
            w.destroy()

        # Apply search filter
        search_filter = self.search_var.get().strip().lower()
        filtered_peers = []
        for p in peers:
            name = (p.get("name") or p.get("id") or "Unknown").lower()
            ip = p.get("ip", "?").lower()
            if not search_filter or search_filter in name or search_filter in ip:
                filtered_peers.append(p)

        self.count_label.configure(text=str(len(filtered_peers)))

        if not filtered_peers:
            # Empty state feedback (No emojis)
            empty_frame = ctk.CTkFrame(self.users_frame, fg_color="transparent")
            empty_frame.pack(fill="both", expand=True, pady=40)

            ctk.CTkLabel(
                empty_frame,
                text="Recherche d'autres appareils...",
                font=ctk.CTkFont(family=FONT_FAMILY, size=13, weight="bold"),
                text_color=THEME["text_muted"],
            ).pack(pady=8)
            return

        for p in filtered_peers:
            card = ctk.CTkFrame(
                self.users_frame,
                fg_color=THEME["bg_subcard"],
                border_width=1,
                border_color=THEME["border"],
                corner_radius=10,
            )
            card.pack(fill="x", padx=4, pady=4)

            name = p.get("name") or p.get("id") or "Unknown"
            ip = p.get("ip", "?")
            port = p.get("tcp_port", 0)
            is_master = p.get("role") == 1

            # Header info
            info_frame = ctk.CTkFrame(card, fg_color="transparent")
            info_frame.pack(fill="x", padx=12, pady=(10, 8))

            text_frame = ctk.CTkFrame(info_frame, fg_color="transparent")
            if self._layout_mode == "vertical":
                text_frame.pack(fill="x", expand=True)
            else:
                text_frame.pack(side="left", fill="both", expand=True)

            ctk.CTkLabel(
                text_frame,
                text=name,
                font=ctk.CTkFont(family=FONT_FAMILY, size=14, weight="bold"),
                text_color=THEME["text_main"],
                anchor="w",
                justify="left",
                wraplength=360 if self._layout_mode == "horizontal" else 260,
            ).pack(anchor="w", fill="x")

            ctk.CTkLabel(
                text_frame,
                text=f"{ip}:{port} - " + ("MASTER" if is_master else "CLIENT"),
                font=ctk.CTkFont(family=FONT_FAMILY, size=11),
                text_color=THEME["success"] if is_master else THEME["text_muted"],
                anchor="w",
                justify="left",
                wraplength=360 if self._layout_mode == "horizontal" else 260,
            ).pack(anchor="w", fill="x", pady=(2, 0))

            # En mode etroit, le bouton descend sous le texte
            # pour eviter que la carte deborde horizontalement.
            btn = ctk.CTkButton(
                info_frame,
                text="Connexion",
                width=100,
                height=30,
                font=ctk.CTkFont(family=FONT_FAMILY, size=12, weight="bold"),
                fg_color=THEME["primary"],
                hover_color=THEME["primary_hover"],
                command=lambda peer=p: self._connect_to_peer(peer),
            )
            if self._layout_mode == "vertical":
                btn.pack(fill="x", pady=(8, 0))
            else:
                btn.pack(side="right", padx=(10, 0))
