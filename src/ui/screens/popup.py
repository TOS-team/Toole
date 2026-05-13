import customtkinter as ctk
from typing import Callable

class ConnectionRequestPopup(ctk.CTkToplevel):
    def __init__(self, master, username: str, ip: str, on_accept: Callable, on_decline: Callable):
        super().__init__(master)
        self.title("Demande de connexion")
        self.geometry("420x220")
        self.resizable(False, False)
        self.attributes("-topmost", True)

        # Icône (à remplacer par une vraie image si tu veux)
        ctk.CTkLabel(self, text="🤝", font=ctk.CTkFont(size=48)).pack(pady=10)

        ctk.CTkLabel(self, text=f"'{username}' veut se connecter avec vous.", 
                    font=ctk.CTkFont(size=16)).pack(pady=5)
        ctk.CTkLabel(self, text=f"IP: {ip}", font=ctk.CTkFont(size=14)).pack()

        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(pady=20)

        ctk.CTkButton(btn_frame, text="Accepter (Entrée)", width=140, height=40,
                     command=lambda: [on_accept(), self.destroy()]).pack(side="left", padx=10)
        
        ctk.CTkButton(btn_frame, text="Refuser (Échap)", width=140, height=40, 
                     fg_color="#FF5252", hover_color="#FF1744",
                     command=lambda: [on_decline(), self.destroy()]).pack(side="left", padx=10)

        self.bind("<Return>", lambda e: [on_accept(), self.destroy()])
        self.bind("<Escape>", lambda e: [on_decline(), self.destroy()])


class FileTransferRequestPopup(ctk.CTkToplevel):
    def __init__(self, master, sender: str, filename: str, size: str, 
                 on_accept: Callable, on_decline: Callable):
        super().__init__(master)
        self.title("Demande de transfert")
        self.geometry("460x240")
        self.resizable(False, False)
        self.attributes("-topmost", True)

        ctk.CTkLabel(self, text="⚠️", font=ctk.CTkFont(size=48)).pack(pady=10)

        ctk.CTkLabel(self, text=f"'{sender}' souhaite vous envoyer :", 
                    font=ctk.CTkFont(size=16)).pack()
        ctk.CTkLabel(self, text=filename, font=ctk.CTkFont(size=18, weight="bold")).pack()
        ctk.CTkLabel(self, text=size, font=ctk.CTkFont(size=14)).pack(pady=5)

        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(pady=20)

        ctk.CTkButton(btn_frame, text="Accepter (Entrée)", width=150, height=45,
                     command=lambda: [on_accept(), self.destroy()]).pack(side="left", padx=15)
        
        ctk.CTkButton(btn_frame, text="Refuser (Échap)", width=150, height=45,
                     fg_color="#FF5252", hover_color="#FF1744",
                     command=lambda: [on_decline(), self.destroy()]).pack(side="left", padx=15)