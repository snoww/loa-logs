!macro NSIS_HOOK_PREINSTALL

!macroend

!macro NSIS_HOOK_POSTINSTALL

!macroend

!macro NSIS_HOOK_PREUNINSTALL

!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete /REBOOTOK "$INSTDIR\WinDivert64.sys"
!macroend