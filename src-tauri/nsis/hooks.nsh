!define TASK_NAME "LOA_Logs_Auto_Start"

!macro NSIS_HOOK_PREINSTALL

!macroend

!macro NSIS_HOOK_POSTINSTALL

!macroend

!macro NSIS_HOOK_PREUNINSTALL

!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete /REBOOTOK "$INSTDIR\WinDivert64.sys"

  ; Remove auto start task if not updating
  nsExec::Exec 'schtasks.exe /query /tn "${TASK_NAME}"'
  Pop $0 ; Return
  ${If} $UpdateMode <> 1
  ${AndIf} $0 == 0
    ExecShellWait 'runas' 'schtasks.exe' '/delete /tn "${TASK_NAME}" /f' SW_HIDE
  ${EndIf}
!macroend