; hooks d'installation NSIS de Toolé : j'ouvre les ports UDP 58199/58200
; dans le pare-feu Windows (profils privé et domaine, jamais public) et je
; retire la règle à la désinstallation. Si l'installateur n'a pas les droits
; admin (mode currentUser), la commande échoue silencieusement : la bannière
; de l'app affiche alors la commande à exécuter manuellement.
!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="Toolé UDP" dir=in action=allow protocol=UDP localport=58199,58200 profile=private,domain'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="Toolé UDP"'
!macroend