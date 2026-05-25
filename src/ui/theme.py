# Hello la BOP, theme centralise pour garder la meme identite visuelle
# dans tous les ecrans sans dupliquer les couleurs.
from CTkMessagebox import CTkMessagebox as _CTkMessagebox

THEME = {
    "bg_dark": "#0B0F19",
    "bg_card": "#152238",
    "bg_subcard": "#1E2D4A",
    "border": "#243B5A",
    "primary": "#6366F1",
    "primary_hover": "#4F46E5",
    "success": "#10B981",
    "success_hover": "#059669",
    "warning": "#F59E0B",
    "danger": "#EF4444",
    "danger_hover": "#DC2626",
    "text_main": "#F3F4F6",
    "text_muted": "#9CA3AF",
    "text_dim": "#6B7280",
}
FONT_FAMILY = "Segoe UI"


def msgbox(title, message, icon="info", **kwargs):
    """Message box avec les couleurs du theme (evite les popups noirs)."""
    return _CTkMessagebox(
        title=title,
        message=message,
        icon=icon,
        fg_color=THEME["bg_card"],
        title_color=THEME["text_main"],
        text_color=THEME["text_main"],
        button_fg_color=THEME["bg_subcard"],
        button_text_color=THEME["text_main"],
        button_hover_color=THEME["border"],
        **kwargs,
    )
