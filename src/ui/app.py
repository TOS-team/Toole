from tkinter import messagebox as tk_messagebox

import customtkinter as ctk
from controller.controller import Controller
from screens.connection import ConnectionScreen
from screens.popup import ConnectionRequestPopup, FileTransferRequestPopup
from screens.transfer import TransferScreen


class App(ctk.CTk):
    def __init__(self):
        super().__init__()

        ctk.set_appearance_mode("dark")
        ctk.set_default_color_theme("blue")

        self.title("Bit Transfer - LAN File Sharing")
        self.geometry("1280x760")
        self.minsize(1100, 680)

        self.controller = Controller()

        # Tabview
        self.tabview = ctk.CTkTabview(self, segmented_button_fg_color="#2B2B2B")
        self.tabview.pack(fill="both", expand=True, padx=15, pady=15)

        self.tabview.add("Connexion")
        self.tabview.add("Transfert")

        # Connexion Screen
        self.connection_screen = ConnectionScreen(
            self.tabview.tab("Connexion"), self.controller
        )
        self.connection_screen.pack(fill="both", expand=True)

        # Transfer Screen
        self.transfer_screen = TransferScreen(
            self.tabview.tab("Transfert"), self.controller
        )
        self.transfer_screen.pack(fill="both", expand=True)

    def show_connection_popup(self, username, ip):
        ConnectionRequestPopup(
            self,
            username=username,
            ip=ip,
            on_accept=lambda: self.controller.accept_connection(username, ip),
            on_decline=lambda: self.controller.decline_connection(username),
        )


if __name__ == "__main__":
    app = App()
    app.mainloop()
