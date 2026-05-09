import customtkinter as ctk
from controller.controller import Controller
from screens.connection import ConnectionScreen
from screens.transfer import TransferScreen
from screens.popup import ConnectionRequestPopup, FileTransferRequestPopup
from tkinter import messagebox as tk_messagebox

class App(ctk.CTk):
    def __init__(self):
        super().__init__()
        ctk.set_appearance_mode("dark")
        ctk.set_default_color_theme("blue")

        self.title("Bit Transfer - LAN File Sharing")
        self.geometry("1200x700")
        self.minsize(1000, 650)

        self.controller = Controller()

        # Tabview
        self.tabview = ctk.CTkTabview(self)
        self.tabview.pack(fill="both", expand=True, padx=20, pady=20)

        self.tabview.add("Connexion")
        self.tabview.add("Transfert")

        self.connection_screen = ConnectionScreen(self.tabview.tab("Connexion"), self.controller)
        self.connection_screen.pack(fill="both", expand=True)

        self.transfer_screen = TransferScreen(self.tabview.tab("Transfert"), self.controller)
        self.transfer_screen.pack(fill="both", expand=True)

        # Test popups (tu peux les appeler depuis des boutons)
        #self.after(2000, self.test_popups)

    # def test_popups(self):
    #     # Exemple de popup connexion
    #     ConnectionRequestPopup(
    #         self,
    #         username="Madeleine_Y",
    #         ip="192.168.1.5",
    #         on_accept=lambda: self.controller.accept_connection("Madeleine_Y", "192.168.1.5"),
    #         on_decline=lambda: self.controller.decline_connection("Madeleine_Y")
    #     )

    #     # Exemple de popup transfert
    #     FileTransferRequestPopup(
    #         self,
    #         sender="Gerard_BIT",
    #         filename="ubuntu.iso",
    #         size="(2.3 Go)",
    #         on_accept=lambda: self.controller.accept_file_transfer("Gerard_BIT", "ubuntu.iso", "2.3 Go"),
    #         on_decline=lambda: print("Transfert refusé")
    #     )


if __name__ == "__main__":
    app = App()
    app.mainloop()
