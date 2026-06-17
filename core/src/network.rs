// Gestion du réseau ad hoc via nmcli (NetworkManager).
//
// detect_wifi_interface() -> String
//   Parse "nmcli device status" pour trouver la première interface WiFi.
//
// create_hotspot(interface, ssid)
//   Crée un hotspot ouvert : nmcli device wifi hotspot ifname <iface> ssid <ssid>
//
// scan_hotspots() -> Vec<String>
//   Liste les réseaux WiFi, filtre ceux commençant par "Toole-".
//
// connect_to_hotspot(ssid)
//   Se connecte au hotspot : nmcli device wifi connect <ssid>
//
// disconnect(interface)
//   Déconnecte l'interface : nmcli device disconnect <iface>
//
// destroy_hotspot(ssid)
//   Supprime la connexion hotspot : nmcli connection delete <ssid>
//
// scan_loop(ui, stop) -> String
//   Boucle de scan toutes les 2s jusqu'à trouver un réseau "Toole-*"
//   ou annulation. Retourne le SSID trouvé.
//
// Gestion privilèges : nmcli → pkexec nmcli → instruction écran
